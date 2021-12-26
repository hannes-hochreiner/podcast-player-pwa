mod tasks;
use super::fetcher::{self};
use crate::objects::*;
use js_sys::ArrayBuffer;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tasks::{
    add_channel_vals::*, add_enclosure::*, add_feed_vals::*, add_item_vals::*, get_all::*,
    get_enclosure::*, get_fetcher_conf::*, get_items_by_channel_id_year_month::*,
    get_items_by_download_ok::*, get_items_by_download_required::*, update_channel::*,
    update_item::*,
};
use uuid::Uuid;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{IdbDatabase, IdbIndexParameters, IdbRequest, IdbTransaction, IdbTransactionMode};
use yew_agent::{Agent, AgentLink, Bridge, Bridged, Context, HandlerId};

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    GetFeeds,
    GetChannels,
    // GetItems,
    GetItemsByChannelIdYearMonth(Uuid, String),
    GetItemsByDownloadRequired,
    GetItemsByDownloadOk,
    AddChannelVals(Vec<ChannelVal>),
    AddItemVals(Vec<ItemVal>),
    AddFeedVals(Vec<FeedVal>),
    DownloadEnclosure(Uuid),
    GetEnclosure(Uuid),
    UpdateChannel(Channel),
    UpdateItem(Item),
    GetFetcherConf(Option<FetcherConfig>),
    AddFeed(String),
}

pub enum Response {
    Error(anyhow::Error),
    Channels(Vec<Channel>),
    Feeds(Vec<Feed>),
    Enclosure(ArrayBuffer),
    AddChannelVals(anyhow::Result<()>),
    AddItemVals(anyhow::Result<()>),
    AddFeedVals(anyhow::Result<()>),
    Items(Vec<Item>),
    Item(Item),
    FetcherConfig(Option<FetcherConfig>),
}

pub struct Repo {
    link: AgentLink<Repo>,
    subscribers: HashSet<HandlerId>,
    open_request: Option<OpenDb>,
    db: Option<IdbDatabase>,
    pending_tasks: Vec<(HandlerId, Box<dyn RepositoryTask>)>,
    in_progress_tasks: HashMap<Uuid, (HandlerId, Box<dyn RepositoryTask>)>,
    fetcher: Box<dyn Bridge<fetcher::Fetcher>>,
    idb_tasks: HashMap<Uuid, IdbResponse>,
    fetcher_tasks: HashMap<Uuid, FetcherTask>,
}

enum FetcherTask {
    DownloadEnclosure(Uuid, HandlerId),
    AddFeed(String, HandlerId),
}

pub enum Msg {
    OpenDbUpdate(web_sys::Event),
    OpenDbSuccess(web_sys::Event),
    IdbRequest((Uuid, Result<web_sys::Event, web_sys::Event>)),
    FetcherMessage(fetcher::Response),
}

pub struct OpenDb {
    _closure_update: Closure<dyn Fn(web_sys::Event)>,
    _closure_success: Closure<dyn Fn(web_sys::Event)>,
    request: web_sys::IdbOpenDbRequest,
}

pub struct IdbResponse {
    request: web_sys::IdbRequest,
    _closure_error: Closure<dyn Fn(web_sys::Event)>,
    _closure_success: Closure<dyn Fn(web_sys::Event)>,
    task_id: Uuid,
}

trait RepositoryTask {
    fn get_request(&mut self, db: &IdbDatabase) -> anyhow::Result<Vec<IdbRequest>>;
    fn set_response(
        &mut self,
        result: Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>,
    ) -> anyhow::Result<Option<Response>>;
    fn create_transaction(
        &self,
        db: &IdbDatabase,
        mode: IdbTransactionMode,
        store_names: &Vec<&str>,
    ) -> anyhow::Result<IdbTransaction> {
        db.transaction_with_str_sequence_and_mode(
            &serde_wasm_bindgen::to_value(&store_names)
                .map_err(|_e| anyhow::anyhow!("error creating store names"))?,
            mode,
        )
        .map_err(|_e| anyhow::anyhow!("error creating transaction"))
    }
}

impl Repo {
    fn init(&mut self) {
        let window: web_sys::Window = web_sys::window().expect("window not available");
        let idb_factory: web_sys::IdbFactory = window.indexed_db().unwrap().unwrap();
        let idb_open_request: web_sys::IdbOpenDbRequest =
            idb_factory.open_with_u32("podcast-player", 1).unwrap();
        let callback_update = self.link.callback(Msg::OpenDbUpdate);
        let callback_success = self.link.callback(Msg::OpenDbSuccess);
        let closure_update =
            Closure::wrap(
                Box::new(move |event: web_sys::Event| callback_update.emit(event))
                    as Box<dyn Fn(_)>,
            );
        idb_open_request.set_onupgradeneeded(Some(closure_update.as_ref().unchecked_ref()));
        let closure_success =
            Closure::wrap(
                Box::new(move |event: web_sys::Event| callback_success.emit(event))
                    as Box<dyn Fn(_)>,
            );
        idb_open_request.set_onsuccess(Some(closure_success.as_ref().unchecked_ref()));

        self.open_request = Some(OpenDb {
            _closure_update: closure_update,
            _closure_success: closure_success,
            request: idb_open_request,
        });
    }

    fn process_tasks(&mut self) {
        match &self.db {
            Some(db) => {
                while self.pending_tasks.len() > 0 {
                    let (handler_id, mut task) = self.pending_tasks.pop().unwrap();

                    match task.get_request(db) {
                        Ok(req) => {
                            let task_id = Uuid::new_v4();
                            self.in_progress_tasks.insert(task_id, (handler_id, task));

                            for r in req {
                                wrap_idb_request(&self.link, &mut self.idb_tasks, task_id, r);
                            }
                        }
                        Err(e) => self.link.respond(handler_id, Response::Error(e)),
                    }
                }
            }
            None => log::error!("no database available"),
        }
    }
}

fn wrap_idb_request(
    link: &AgentLink<Repo>,
    active_idb_tasks: &mut HashMap<Uuid, IdbResponse>,
    task_id: Uuid,
    request: IdbRequest,
) {
    let request_id = Uuid::new_v4();
    let callback_success = link.callback(Msg::IdbRequest);
    let callback_error = link.callback(Msg::IdbRequest);
    let closure_success = Closure::wrap(Box::new(move |event: web_sys::Event| {
        callback_success.emit((request_id, Ok(event)))
    }) as Box<dyn Fn(_)>);
    let closure_error = Closure::wrap(Box::new(move |event: web_sys::Event| {
        callback_error.emit((request_id, Err(event)))
    }) as Box<dyn Fn(_)>);
    request.set_onsuccess(Some(closure_error.as_ref().unchecked_ref()));
    request.set_onerror(Some(closure_error.as_ref().unchecked_ref()));
    active_idb_tasks.insert(
        request_id,
        IdbResponse {
            request,
            _closure_error: closure_error,
            _closure_success: closure_success,
            task_id,
        },
    );
}

impl Agent for Repo {
    type Reach = Context<Self>;
    type Message = Msg;
    type Input = Request;
    type Output = Response;

    fn create(link: AgentLink<Self>) -> Self {
        let fetcher_cb = link.callback(Msg::FetcherMessage);

        let mut obj = Self {
            link,
            subscribers: HashSet::new(),
            open_request: None,
            db: None,
            pending_tasks: Vec::new(),
            fetcher: fetcher::Fetcher::bridge(fetcher_cb),
            fetcher_tasks: HashMap::new(),
            idb_tasks: HashMap::new(),
            in_progress_tasks: HashMap::new(),
        };

        obj.init();

        obj
    }

    fn update(&mut self, msg: Self::Message) {
        match msg {
            Msg::FetcherMessage(resp) => match resp {
                fetcher::Response::Binary(uuid, res) => {
                    let task = self.fetcher_tasks.remove(&uuid).unwrap();

                    match task {
                        FetcherTask::DownloadEnclosure(item_id, handler_id) => {
                            match (&self.db, res) {
                                (Some(_db), Ok(data)) => {
                                    self.pending_tasks.push((
                                        handler_id,
                                        Box::new(AddEnclosureTask::new_with_item_id_data(
                                            item_id, data,
                                        )),
                                    ));
                                    self.process_tasks();
                                }
                                (None, _) => log::error!("could not find database"),
                                (_, Err(e)) => log::error!("error downloading enclosure: {}", e),
                            }
                        }
                        _ => {}
                    }
                }
                fetcher::Response::Text(uuid, _res) => {
                    let task = self.fetcher_tasks.remove(&uuid).unwrap();

                    match task {
                        _ => {}
                    }
                }
            },
            Msg::OpenDbUpdate(_e) => {
                let idb_db = IdbDatabase::from(
                    self.open_request
                        .as_ref()
                        .unwrap()
                        .request
                        .result()
                        .unwrap(),
                );
                let object_stores = vec![
                    "channels",
                    "feeds",
                    "items",
                    "enclosures",
                    "images",
                    "images-meta",
                    "configuration",
                ];
                let mut indices = HashMap::new();
                indices.insert(
                    "items",
                    vec![
                        (
                            "channel_id_year_month",
                            vec!["val.channel_id", "keys.year_month"],
                        ),
                        ("download_required", vec!["keys.download_required"]),
                        ("download_ok", vec!["keys.download_ok"]),
                    ],
                );

                for object_store in object_stores {
                    match idb_db.create_object_store(object_store) {
                        Ok(os) => {
                            if indices.contains_key(object_store) {
                                for (name, key_paths) in &indices[object_store] {
                                    match os.create_index_with_str_sequence_and_optional_parameters(
                                        name,
                                        &serde_wasm_bindgen::to_value(key_paths).unwrap(),
                                        &IdbIndexParameters::new(),
                                    ) {
                                        Ok(_) => log::info!("created index {}", name),
                                        Err(e) => log::error!("failed to create index: {:?}", e),
                                    };
                                }
                            }
                        }
                        Err(e) => {
                            log::error!(
                                "failed to create object store \"{}\": {:?}",
                                object_store,
                                e
                            );
                        }
                    }
                }
            }
            Msg::OpenDbSuccess(_e) => {
                self.db = Some(
                    self.open_request
                        .as_ref()
                        .unwrap()
                        .request
                        .result()
                        .unwrap()
                        .into(),
                );
                self.open_request = None;
                log::info!("open db");
                self.process_tasks();
            }
            Msg::IdbRequest(res) => {
                let req = self.idb_tasks.remove(&res.0).unwrap();
                let (handler_id, mut task) = self.in_progress_tasks.remove(&req.task_id).unwrap();

                match task.set_response(req.request.result()) {
                    Ok(Some(resp)) => match resp {
                        Response::Item(item) => {
                            for sub in &self.subscribers {
                                self.link.respond(*sub, Response::Item(item.clone()));
                            }
                        }
                        _ => self.link.respond(handler_id, resp),
                    },
                    Ok(None) => {
                        self.in_progress_tasks
                            .insert(req.task_id, (handler_id, task));
                    }
                    Err(e) => self.link.respond(handler_id, Response::Error(e)),
                }
            }
        }
    }

    fn handle_input(&mut self, msg: Self::Input, handler_id: HandlerId) {
        match msg {
            Request::AddFeed(url) => {
                let task_id = Uuid::new_v4();

                self.fetcher_tasks
                    .insert(task_id, FetcherTask::AddFeed(url.clone(), handler_id));
                self.fetcher.send(fetcher::Request::PostString(
                    task_id,
                    format!("/api/feeds"),
                    url,
                ));
            }
            Request::GetFeeds => self
                .pending_tasks
                .push((handler_id, Box::new(GetAllTask::new(Kind::Feed)))),
            Request::GetFetcherConf(fct) => self.pending_tasks.push((
                handler_id,
                Box::new(GetFetcherConfTask::new_with_option(fct)),
            )),
            Request::AddFeedVals(feeds) => self.pending_tasks.push((
                handler_id,
                Box::new(AddFeedValsTask::new_with_feed_vals(feeds)),
            )),
            Request::AddChannelVals(channels) => self.pending_tasks.push((
                handler_id,
                Box::new(AddChannelValsTask::new_with_channel_vals(channels)),
            )),
            Request::AddItemVals(items) => self.pending_tasks.push((
                handler_id,
                Box::new(AddItemValsTask::new_with_item_vals(items)),
            )),
            Request::GetChannels => self
                .pending_tasks
                .push((handler_id, Box::new(GetAllTask::new(Kind::Channel)))),
            Request::GetEnclosure(id) => self
                .pending_tasks
                .push((handler_id, Box::new(GetEnclosureTask::new_with_id(id)))),
            Request::UpdateChannel(channel) => self.pending_tasks.push((
                handler_id,
                Box::new(UpdateChannelTask::new_with_channel(channel)),
            )),
            Request::GetItemsByChannelIdYearMonth(channel_id, year_month) => {
                self.pending_tasks.push((
                    handler_id,
                    Box::new(
                        GetItemsByChannelIdYearMonthTask::new_with_channel_id_year_month(
                            channel_id, year_month,
                        ),
                    ),
                ))
            }
            Request::UpdateItem(item) => self
                .pending_tasks
                .push((handler_id, Box::new(UpdateItemTask::new_with_item(item)))),
            Request::GetItemsByDownloadRequired => self
                .pending_tasks
                .push((handler_id, Box::new(GetItemsByDownloadRequiredTask::new()))),
            Request::GetItemsByDownloadOk => self
                .pending_tasks
                .push((handler_id, Box::new(GetItemsByDownloadOkTask::new()))),
            Request::DownloadEnclosure(item_id) => {
                let task_id = Uuid::new_v4();

                self.fetcher_tasks
                    .insert(task_id, FetcherTask::DownloadEnclosure(item_id, handler_id));
                self.fetcher.send(fetcher::Request::FetchBinary(
                    task_id,
                    format!("/api/items/{}/stream", item_id),
                ));
            }
        }

        match &self.db {
            Some(_) => self.process_tasks(),
            None => log::warn!("database not available; postponing task"),
        }
    }

    fn connected(&mut self, id: HandlerId) {
        self.subscribers.insert(id);
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
    }
}
