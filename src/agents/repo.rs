mod task;
use super::{fetcher, notifier};
use crate::objects::*;
use chrono::{DateTime, FixedOffset};
use js_sys::ArrayBuffer;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use task::*;
use uuid::Uuid;
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{IdbDatabase, IdbRequest, IdbTransaction, IdbTransactionMode};
use yew_agent::{Agent, AgentLink, Bridge, Bridged, Context, Dispatched, Dispatcher, HandlerId};

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    GetFeeds,    // returns Feeds only to requester
    GetChannels, // returns Channels only to requester
    GetYearMonthKeysByChannelId(Uuid),
    GetItemsByChannelIdYearMonth(Uuid, String), // returns Items only to requester
    GetItemsByDownloadRequired,                 // returns Items only to requester
    GetItemsByDownloadOk,                       // returns Items only to requester
    GetEnclosure(Uuid),                         // returns Enclosure only to the requester
    GetChannel(Uuid),
    DeleteEnclosure(Item),  // returns UpdatedItem to all subscribers
    UpdateChannel(Channel), // returns UpdatedChannel to all subscribers
    UpdateItem(Item),       // returns UpdatedItem to all subscribers
    GetFetcherConf(Option<FetcherConfig>), // returns FetcherConfig only to requester
    GetUpdaterConf(Option<UpdaterConfig>), // returns UpdaterConfig only to requester
    AddFeed(String),
}

#[derive(Debug, Clone)]
pub enum Response {
    Feeds(Vec<FeedVal>),
    Channels(Vec<Channel>),
    YearMonthKeys(Vec<String>),
    Items(Vec<Item>),
    Enclosure(ArrayBuffer),
    UpdatedFeed(FeedVal),
    UpdatedChannel(Channel),
    UpdatedItem(Item),
    FetcherConfig(Option<FetcherConfig>),
    UpdaterConfig(Option<UpdaterConfig>),
}

pub struct Repo {
    link: AgentLink<Repo>,
    subscribers: HashSet<HandlerId>,
    db: Option<IdbDatabase>,
    fetcher: Box<dyn Bridge<fetcher::Fetcher>>,
    notifier: Dispatcher<notifier::Notifier>,
    tasks: Vec<Task>,
    idb_closure_error: Closure<dyn Fn(web_sys::Event)>,
    idb_closure_success: Closure<dyn Fn(web_sys::Event)>,
    idb_closure_trans_complete: Closure<dyn Fn(web_sys::Event)>,
    idb_closure_trans_abort: Closure<dyn Fn(web_sys::Event)>,
    idb_closure_trans_error: Closure<dyn Fn(web_sys::Event)>,
    _closure_interval: Closure<dyn Fn(web_sys::Event)>,
}

#[derive(Debug)]
pub enum Message {
    OpenDbUpdate(web_sys::Event),
    OpenDbResult(Result<web_sys::Event, web_sys::Event>),
    IdbRequest(Result<web_sys::Event, web_sys::Event>),
    IdbTransaction(Result<web_sys::Event, web_sys::Event>),
    FetcherMessage(fetcher::Response),
    Interval(web_sys::Event),
}

trait RepositoryTask {
    fn get_request(&mut self, db: &IdbDatabase) -> Result<Vec<IdbRequest>, JsError>;
    fn set_response(
        &mut self,
        result: Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>,
    ) -> Result<Option<Response>, JsError>;
    fn create_transaction(
        &self,
        db: &IdbDatabase,
        mode: IdbTransactionMode,
        store_names: &Vec<&str>,
    ) -> Result<IdbTransaction, JsError> {
        db.transaction_with_str_sequence_and_mode(
            &serde_wasm_bindgen::to_value(&store_names)?,
            mode,
        )
        .map_err(Into::into)
    }
}

impl Repo {
    fn process_tasks(&mut self) {
        if let Some(mut task) = self.tasks.pop() {
            match self.process_task(&mut task) {
                Ok(response) => match response {
                    true => self.process_tasks(),
                    false => self.tasks.push(task),
                },
                Err(e) => {
                    self.notifier.send(notifier::Request::NotifyError(e));
                    self.process_tasks();
                }
            }
        }
    }

    fn process_task(&mut self, task: &mut Task) -> Result<bool, JsError> {
        // log::info!("process task: {:?}", task);
        match task {
            Task::OpenDb(task) => self.process(task),
            Task::GetAll(task) => self.process(task),
            Task::PutGetWithKey(task) => self.process(task),
            Task::StoreEnclosure(task) => self.process(task),
            Task::DownloadStarted(task) => self.process(task),
            Task::DeleteEnclosure(task) => self.process(task),
            Task::SyncVal(task) => self.process(task),
            Task::GetKeys(task) => self.process(task),
        }
    }

    fn process_update(&mut self, msg: Message) -> Result<(), JsError> {
        match msg {
            Message::OpenDbUpdate(_) => {
                if let Some(mut task) = self.tasks.last_mut() {
                    match &mut task {
                        Task::OpenDb(task) => {
                            task.set_update_requested();
                        }
                        _ => {}
                    }
                }
            }
            Message::OpenDbResult(_) => {
                if let Some(mut task) = self.tasks.last_mut() {
                    match &mut task {
                        Task::OpenDb(task) => task.request_completed(),
                        _ => {}
                    }
                }
            }
            Message::FetcherMessage(resp) => match resp {
                fetcher::Response::PullFeedVals(feed_vals) => {
                    for feed_val in feed_vals {
                        self.tasks.insert(
                            0,
                            Task::SyncVal(task::sync_val::Task::new(task::sync_val::Value::Feed(
                                feed_val,
                            ))),
                        );
                    }
                }
                fetcher::Response::PullChannelVals(channel_vals) => {
                    for channel_val in channel_vals {
                        self.tasks.insert(
                            0,
                            Task::SyncVal(task::sync_val::Task::new(
                                task::sync_val::Value::Channel(channel_val),
                            )),
                        );
                    }
                }
                fetcher::Response::PullItemVals(item_vals) => {
                    for item_val in item_vals {
                        self.tasks.insert(
                            0,
                            Task::SyncVal(task::sync_val::Task::new(task::sync_val::Value::Item(
                                item_val,
                            ))),
                        );
                    }
                }
                fetcher::Response::PullDownload(item_id, data) => self.tasks.insert(
                    0,
                    Task::StoreEnclosure(task::store_enclosure::Task::new(item_id, data)),
                ),
                fetcher::Response::PullDownloadStarted(item_id) => {
                    self.tasks.insert(
                        0,
                        Task::DownloadStarted(task::download_started::Task::new(item_id)),
                    );
                }
                fetcher::Response::Binary(task_id, res) => {}
                fetcher::Response::Text(task_id, res) => {}
            },
            Message::Interval(_) => {
                // self.tasks.insert(
                //     0,
                //     Task::GetUniqueKeys(task::get_unique_keys::Task::new(
                //         task::get_unique_keys::Kind::Item,
                //     )),
                // );
                self.tasks.insert(
                    0,
                    Task::GetKeys(task::get_keys::Task::new(task::get_keys::Kind::LastUpdate(
                        task::get_keys::ObjectKind::Feed,
                    ))),
                );
                self.tasks.insert(
                    0,
                    Task::GetKeys(task::get_keys::Task::new(task::get_keys::Kind::LastUpdate(
                        task::get_keys::ObjectKind::Channel,
                    ))),
                );
                self.tasks.insert(
                    0,
                    Task::GetKeys(task::get_keys::Task::new(task::get_keys::Kind::LastUpdate(
                        task::get_keys::ObjectKind::Item,
                    ))),
                );
                // self.fetcher.send(fetcher::Request::PullFeedVals(None));
                // self.fetcher.send(fetcher::Request::PullChannelVals(None));
                // self.fetcher.send(fetcher::Request::PullItemVals(None));
                self.tasks.insert(
                    0,
                    Task::GetAll(task::get_all::Task::new(
                        None,
                        task::get_all::Kind::ItemDownloadRequired,
                        Some(serde_wasm_bindgen::to_value(&vec!["true"])?),
                        Some("download_required".into()),
                    )),
                )
            }
            Message::IdbTransaction(_) => {
                if let Some(task) = self.tasks.last_mut() {
                    task.transaction_complete();
                }
            }
            Message::IdbRequest(_) => {}
        }
        // match msg {
        //     Message::FetcherMessage(resp) => match resp {
        //         fetcher::Response::Binary(uuid, res) => {
        //             let task = self.fetcher_tasks.remove(&uuid).ok_or("task not found")?;

        //             match task {
        //                 FetcherTask::DownloadEnclosure(item_id, handler_id) => {
        //                     match (&self.db, res) {
        //                         (Some(_db), Ok(data)) => {
        //                             self.pending_tasks.push((
        //                                 handler_id,
        //                                 Box::new(AddEnclosureTask::new_with_item_id_data(
        //                                     item_id, data,
        //                                 )),
        //                             ));
        //                             self.process_tasks()?;
        //                         }
        //                         (None, _) => log::error!("could not find database"),
        //                         (_, Err(e)) => log::error!("error downloading enclosure: {}", e),
        //                     }
        //                 }
        //                 _ => {}
        //             }
        //         }
        //         fetcher::Response::Text(uuid, _res) => {
        //             let task = self.fetcher_tasks.remove(&uuid).ok_or("task not found")?;

        //             match task {
        //                 _ => {}
        //             }
        //         }
        //     },
        //     Message::OpenDbUpdate(_e) => {
        //         let idb_db = IdbDatabase::from(
        //             self.open_request
        //                 .as_ref()
        //                 .ok_or("could not get reference")?
        //                 .request
        //                 .result()?,
        //         );
        //         let object_stores = vec![
        //             "channels",
        //             "feeds",
        //             "items",
        //             "enclosures",
        //             "images",
        //             "images-meta",
        //             "configuration",
        //         ];
        //         let mut indices = HashMap::new();
        //         indices.insert(
        //             "items",
        //             vec![
        //                 (
        //                     "channel_id_year_month",
        //                     vec!["val.channel_id", "keys.year_month"],
        //                 ),
        //                 ("download_required", vec!["keys.download_required"]),
        //                 ("download_ok", vec!["keys.download_ok"]),
        //             ],
        //         );

        //         for object_store in object_stores {
        //             match idb_db.create_object_store(object_store) {
        //                 Ok(os) => {
        //                     if indices.contains_key(object_store) {
        //                         for (name, key_paths) in &indices[object_store] {
        //                             match os.create_index_with_str_sequence_and_optional_parameters(
        //                                 name,
        //                                 &serde_wasm_bindgen::to_value(key_paths)?,
        //                                 &IdbIndexParameters::new(),
        //                             ) {
        //                                 Ok(_) => log::info!("created index {}", name),
        //                                 Err(e) => log::error!("failed to create index: {:?}", e),
        //                             };
        //                         }
        //                     }
        //                 }
        //                 Err(e) => {
        //                     log::error!(
        //                         "failed to create object store \"{}\": {:?}",
        //                         object_store,
        //                         e
        //                     );
        //                 }
        //             }
        //         }
        //     }
        //     Message::OpenDbSuccess(_e) => {
        //         self.db = Some(
        //             self.open_request
        //                 .as_ref()
        //                 .ok_or("could not get reference")?
        //                 .request
        //                 .result()?
        //                 .into(),
        //         );
        //         self.open_request = None;
        //         log::info!("open db");
        //         self.process_tasks()?;
        //     }
        //     Message::IdbRequest(res) => {
        //         let req = self.idb_tasks.remove(&res.0).ok_or("task not found")?;
        //         let (handler_id, mut task) = self
        //             .in_progress_tasks
        //             .remove(&req.task_id)
        //             .ok_or("task not found")?;

        //         match task.set_response(req.request.result()) {
        //             Ok(Some(resp)) => {
        //                 self.link.respond(handler_id.clone(), resp.clone());
        //                 match resp {
        //                     Response::UpdateItem(item) | Response::DownloadEnclosure(item) => {
        //                         for sub in &self.subscribers {
        //                             if sub.is_respondable() && *sub != handler_id {
        //                                 self.link.respond(
        //                                     *sub,
        //                                     Response::ModifiedItems(vec![item.clone()]),
        //                                 );
        //                             }
        //                         }
        //                     }
        //                     _ => {}
        //                 }
        //             }
        //             Ok(None) => {
        //                 self.in_progress_tasks
        //                     .insert(req.task_id, (handler_id, task));
        //             }
        //             Err(e) => self.link.respond(handler_id, Response::Error(e)),
        //         }
        //     }
        // }

        Ok(())
    }

    fn process_handle_input(&mut self, msg: Request, handler_id: HandlerId) -> Result<(), JsError> {
        match msg {
            Request::AddFeed(_) => {
                todo!()
            }
            Request::GetChannel(channel_id) => self.tasks.insert(
                0,
                Task::PutGetWithKey(task::put_get_with_key::Task::new(
                    Some(handler_id),
                    task::put_get_with_key::Kind::Channel,
                    serde_wasm_bindgen::to_value(&channel_id)?,
                    None,
                )),
            ),
            Request::GetYearMonthKeysByChannelId(channel_id) => self.tasks.insert(
                0,
                Task::GetKeys(task::get_keys::Task::new(
                    task::get_keys::Kind::ItemYearMonth {
                        handler_id,
                        channel_id,
                    },
                )),
            ),
            Request::GetEnclosure(id) => self.tasks.insert(
                0,
                Task::PutGetWithKey(task::put_get_with_key::Task::new(
                    Some(handler_id),
                    task::put_get_with_key::Kind::Enclosure,
                    serde_wasm_bindgen::to_value(&id)?,
                    None,
                )),
            ),
            Request::DeleteEnclosure(item) => self.tasks.insert(
                0,
                Task::DeleteEnclosure(task::delete_enclosure::Task::new(item)),
            ),
            Request::GetItemsByDownloadOk => self.tasks.insert(
                0,
                Task::GetAll(task::get_all::Task::new(
                    Some(handler_id),
                    task::get_all::Kind::Item,
                    Some(serde_wasm_bindgen::to_value(&vec![String::from("true")])?),
                    Some(String::from("download_ok")),
                )),
            ),
            Request::GetFetcherConf(value) => self.tasks.insert(
                0,
                Task::PutGetWithKey(task::put_get_with_key::Task::new(
                    None,
                    task::put_get_with_key::Kind::Configuration,
                    serde_wasm_bindgen::to_value("fetcher")?,
                    match &value {
                        Some(value) => Some(serde_wasm_bindgen::to_value(value)?),
                        None => None,
                    },
                )),
            ),
            Request::GetUpdaterConf(value) => self.tasks.insert(
                0,
                Task::PutGetWithKey(task::put_get_with_key::Task::new(
                    None,
                    task::put_get_with_key::Kind::Configuration,
                    serde_wasm_bindgen::to_value("updater")?,
                    match &value {
                        Some(value) => Some(serde_wasm_bindgen::to_value(value)?),
                        None => None,
                    },
                )),
            ),
            Request::UpdateItem(value) => self.tasks.insert(
                0,
                Task::PutGetWithKey(task::put_get_with_key::Task::new(
                    None,
                    task::put_get_with_key::Kind::Item,
                    serde_wasm_bindgen::to_value(&value.get_id())?,
                    Some(serde_wasm_bindgen::to_value(&value)?),
                )),
            ),
            Request::UpdateChannel(value) => self.tasks.insert(
                0,
                Task::PutGetWithKey(task::put_get_with_key::Task::new(
                    None,
                    task::put_get_with_key::Kind::Channel,
                    serde_wasm_bindgen::to_value(&value.val.id)?,
                    Some(serde_wasm_bindgen::to_value(&value)?),
                )),
            ),
            Request::GetFeeds => self.tasks.insert(
                0,
                Task::GetAll(task::get_all::Task::new(
                    Some(handler_id),
                    task::get_all::Kind::Feed,
                    None,
                    None,
                )),
            ),
            Request::GetChannels => self.tasks.insert(
                0,
                Task::GetAll(task::get_all::Task::new(
                    Some(handler_id),
                    task::get_all::Kind::Channel,
                    None,
                    None,
                )),
            ),
            Request::GetItemsByChannelIdYearMonth(channel_id, year_month) => self.tasks.insert(
                0,
                Task::GetAll(task::get_all::Task::new(
                    Some(handler_id),
                    task::get_all::Kind::Item,
                    Some(serde_wasm_bindgen::to_value(&vec![
                        channel_id.to_string(),
                        year_month,
                    ])?),
                    Some("channel_id_year_month".into()),
                )),
            ),
            Request::GetItemsByDownloadRequired => self.tasks.insert(
                0,
                Task::GetAll(task::get_all::Task::new(
                    Some(handler_id),
                    task::get_all::Kind::Item,
                    Some(serde_wasm_bindgen::to_value(&vec!["true"])?),
                    Some("download_required".into()),
                )),
            ),
        }

        Ok(())
    }
}

impl Agent for Repo {
    type Reach = Context<Self>;
    type Message = Message;
    type Input = Request;
    type Output = Response;

    fn create(link: AgentLink<Self>) -> Self {
        let fetcher_cb = link.callback(Message::FetcherMessage);
        let mut notifier = notifier::Notifier::dispatcher();
        let idb_callback_success = link.callback(Message::IdbRequest);
        let idb_callback_error = link.callback(Message::IdbRequest);
        let idb_callback_trans_error = link.callback(Message::IdbTransaction);
        let idb_callback_trans_complete = link.callback(Message::IdbTransaction);
        let idb_callback_trans_abort = link.callback(Message::IdbTransaction);
        let idb_closure_success = Closure::wrap(Box::new(move |event: web_sys::Event| {
            idb_callback_success.emit(Ok(event))
        }) as Box<dyn Fn(_)>);
        let idb_closure_error =
            Closure::wrap(
                Box::new(move |event: web_sys::Event| idb_callback_error.emit(Err(event)))
                    as Box<dyn Fn(_)>,
            );
        let idb_closure_trans_error = Closure::wrap(Box::new(move |event: web_sys::Event| {
            idb_callback_trans_error.emit(Err(event))
        }) as Box<dyn Fn(_)>);
        let idb_closure_trans_abort = Closure::wrap(Box::new(move |event: web_sys::Event| {
            idb_callback_trans_abort.emit(Err(event))
        }) as Box<dyn Fn(_)>);
        let idb_closure_trans_complete = Closure::wrap(Box::new(move |event: web_sys::Event| {
            idb_callback_trans_complete.emit(Ok(event))
        }) as Box<dyn Fn(_)>);
        let callback_interval = link.callback(Message::Interval);
        let closure_interval =
            Closure::wrap(
                Box::new(move |event: web_sys::Event| callback_interval.emit(event))
                    as Box<dyn Fn(_)>,
            );
        match web_sys::window()
            .ok_or(JsError::from_str("could not obtain window"))
            .and_then(|w| {
                w.set_interval_with_callback_and_timeout_and_arguments(
                    closure_interval.as_ref().unchecked_ref(),
                    15_000,
                    &js_sys::Array::new(),
                )
                .map_err(Into::into)
            }) {
            Ok(_handle) => {}
            Err(e) => notifier.send(notifier::Request::NotifyError(e)),
        }

        let mut obj = Self {
            link,
            subscribers: HashSet::new(),
            db: None,
            fetcher: fetcher::Fetcher::bridge(fetcher_cb),
            notifier,
            tasks: Vec::new(),
            idb_closure_error,
            idb_closure_success,
            _closure_interval: closure_interval,
            idb_closure_trans_abort,
            idb_closure_trans_complete,
            idb_closure_trans_error,
        };

        obj.tasks.insert(0, Task::OpenDb(open_db::Task::new()));
        obj.process_tasks();
        obj
    }

    fn update(&mut self, msg: Self::Message) {
        // log::info!("update: {:?}", msg);
        match self.process_update(msg) {
            Ok(()) => {}
            Err(e) => self.notifier.send(notifier::Request::NotifyError(e)),
        }

        self.process_tasks();
    }

    fn handle_input(&mut self, msg: Self::Input, handler_id: HandlerId) {
        // log::info!("handle_input: {:?}", msg);
        match self.process_handle_input(msg, handler_id) {
            Ok(()) => {}
            Err(e) => self.notifier.send(notifier::Request::NotifyError(e)),
        }

        self.process_tasks();
        // match msg {
        //     Request::GetUpdaterConf(conf) => self.pending_tasks.push((
        //         handler_id,
        //         Box::new(put_get_with_key::PutGetWithKeyTask::new(
        //             put_get_with_key::Kind::ConfigurationUpdater,
        //             conf.map(|c| serde_wasm_bindgen::to_value(&c).unwrap()),
        //             serde_wasm_bindgen::to_value("updater").unwrap(),
        //         )),
        //     )),
        //     Request::GetFetcherConf(conf) => self.pending_tasks.push((
        //         handler_id,
        //         Box::new(put_get_with_key::PutGetWithKeyTask::new(
        //             put_get_with_key::Kind::ConfigurationFetcher,
        //             conf.map(|c| serde_wasm_bindgen::to_value(&c).unwrap()),
        //             serde_wasm_bindgen::to_value("fetcher").unwrap(),
        //         )),
        //     )),
        //     Request::AddFeed(url) => {
        //         let task_id = Uuid::new_v4();

        //         self.fetcher_tasks
        //             .insert(task_id, FetcherTask::AddFeed(url.clone(), handler_id));
        //         self.fetcher.send(fetcher::Request::PostString(
        //             task_id,
        //             format!("/api/feeds"),
        //             url,
        //         ));
        //     }
        //     Request::GetFeeds => self
        //         .pending_tasks
        //         .push((handler_id, Box::new(GetAllTask::new(Kind::Feeds, None)))),
        //     Request::AddFeedVals(feeds) => self.pending_tasks.push((
        //         handler_id,
        //         Box::new(AddFeedValsTask::new_with_feed_vals(feeds)),
        //     )),
        //     Request::AddChannelVals(channels) => self.pending_tasks.push((
        //         handler_id,
        //         Box::new(AddChannelValsTask::new_with_channel_vals(channels)),
        //     )),
        //     Request::AddItemVals(items) => self.pending_tasks.push((
        //         handler_id,
        //         Box::new(AddItemValsTask::new_with_item_vals(items)),
        //     )),
        //     Request::GetChannels => self
        //         .pending_tasks
        //         .push((handler_id, Box::new(GetAllTask::new(Kind::Channels, None)))),
        //     Request::GetEnclosure(id) => self
        //         .pending_tasks
        //         .push((handler_id, Box::new(GetEnclosureTask::new_with_id(id)))),
        //     Request::UpdateChannel(channel) => self.pending_tasks.push((
        //         handler_id,
        //         Box::new(UpdateChannelTask::new_with_channel(channel)),
        //     )),
        //     Request::GetItemsByChannelIdYearMonth(channel_id, year_month) => {
        //         self.pending_tasks.push((
        //             handler_id,
        //             Box::new(
        //                 GetItemsByChannelIdYearMonthTask::new_with_channel_id_year_month(
        //                     channel_id, year_month,
        //                 ),
        //             ),
        //         ))
        //     }
        //     Request::UpdateItem(item) => self
        //         .pending_tasks
        //         .push((handler_id, Box::new(UpdateItemTask::new_with_item(item)))),
        //     Request::GetItemsByDownloadRequired => self
        //         .pending_tasks
        //         .push((handler_id, Box::new(GetItemsByDownloadRequiredTask::new()))),
        //     Request::GetItemsByDownloadOk => self
        //         .pending_tasks
        //         .push((handler_id, Box::new(GetItemsByDownloadOkTask::new()))),
        //     Request::DownloadEnclosure(item_id) => {
        //         let task_id = Uuid::new_v4();

        //         self.fetcher_tasks
        //             .insert(task_id, FetcherTask::DownloadEnclosure(item_id, handler_id));
        //         self.fetcher.send(fetcher::Request::FetchBinary(
        //             task_id,
        //             format!("/api/items/{}/stream", item_id),
        //         ));
        //     }
        // }
    }

    fn connected(&mut self, id: HandlerId) {
        self.subscribers.insert(id);
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
    }
}
