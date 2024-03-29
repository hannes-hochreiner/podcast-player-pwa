mod task;
use super::{fetcher, notifier};
use crate::{objects::*, utils};
use js_sys::ArrayBuffer;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use task::*;
use uuid::Uuid;
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{ConnectionType, IdbDatabase, IdbRequest, IdbTransaction, IdbTransactionMode};
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

                Ok(())
            }
            Message::OpenDbResult(_) => {
                if let Some(mut task) = self.tasks.last_mut() {
                    match &mut task {
                        Task::OpenDb(task) => task.request_completed(),
                        _ => {}
                    }
                }

                Ok(())
            }
            Message::FetcherMessage(resp) => match resp {
                fetcher::Response::PullFeedVals(feed_vals) => match feed_vals {
                    Ok(feed_vals) => {
                        for feed_val in feed_vals {
                            self.tasks.insert(
                                0,
                                Task::SyncVal(task::sync_val::Task::new(
                                    task::sync_val::Value::Feed(feed_val),
                                )),
                            );
                        }

                        Ok(())
                    }
                    Err(e) => match utils::get_connection_type()? {
                        ConnectionType::None => Ok(()),
                        _ => Err(e),
                    },
                },
                fetcher::Response::PullChannelVals(channel_vals) => match channel_vals {
                    Ok(channel_vals) => {
                        for channel_val in channel_vals {
                            self.tasks.insert(
                                0,
                                Task::SyncVal(task::sync_val::Task::new(
                                    task::sync_val::Value::Channel(channel_val),
                                )),
                            );
                        }

                        Ok(())
                    }
                    Err(e) => match utils::get_connection_type()? {
                        ConnectionType::None => Ok(()),
                        _ => Err(e),
                    },
                },
                fetcher::Response::PullItemVals(item_vals) => match item_vals {
                    Ok(item_vals) => {
                        for item_val in item_vals {
                            self.tasks.insert(
                                0,
                                Task::SyncVal(task::sync_val::Task::new(
                                    task::sync_val::Value::Item(item_val),
                                )),
                            );
                        }

                        Ok(())
                    }
                    Err(e) => match utils::get_connection_type()? {
                        ConnectionType::None => Ok(()),
                        _ => Err(e),
                    },
                },
                fetcher::Response::PullDownload(item_id, data) => match data {
                    Ok(data) => {
                        self.tasks.insert(
                            0,
                            Task::StoreEnclosure(task::store_enclosure::Task::new(item_id, data)),
                        );

                        Ok(())
                    }
                    Err(e) => match utils::get_connection_type()? {
                        ConnectionType::None => Ok(()),
                        _ => Err(e),
                    },
                },
                fetcher::Response::PullDownloadStarted(item_id) => {
                    self.tasks.insert(
                        0,
                        Task::DownloadStarted(task::download_started::Task::new(item_id)),
                    );

                    Ok(())
                }
                fetcher::Response::Binary(_task_id, _res) => Ok(()),
                fetcher::Response::Text(_task_id, _res) => Ok(()),
            },
            Message::Interval(_) => {
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

                self.tasks.insert(
                    0,
                    Task::GetAll(task::get_all::Task::new(
                        None,
                        task::get_all::Kind::ItemDownloadRequired,
                        Some(serde_wasm_bindgen::to_value(&vec!["true"])?),
                        Some("download_required".into()),
                    )),
                );

                Ok(())
            }
            Message::IdbTransaction(_) => {
                if let Some(task) = self.tasks.last_mut() {
                    task.transaction_complete();
                }

                Ok(())
            }
            Message::IdbRequest(_) => Ok(()),
        }
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
    }

    fn connected(&mut self, id: HandlerId) {
        self.subscribers.insert(id);
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
    }
}
