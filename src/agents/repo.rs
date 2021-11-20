use super::fetcher;
use crate::objects::channel::Channel;
use anyhow::Context as AnyhowContext;
use js_sys::ArrayBuffer;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{IdbDatabase, IdbTransactionMode};
use yew::worker::*;

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    GetChannels,
    AddChannels(Vec<Channel>),
    DownloadEnclosure(Uuid),
    GetEnclosure(Uuid),
}

pub enum Response {
    Channels(anyhow::Result<Vec<Channel>>),
    Enclosure(anyhow::Result<ArrayBuffer>),
}

pub struct Repo {
    link: AgentLink<Repo>,
    subscribers: HashSet<HandlerId>,
    open_request: Option<OpenDb>,
    db: Option<IdbDatabase>,
    idb_request: Option<IdbReq>,
    pending_tasks: Vec<Task>,
    fetcher: Box<dyn Bridge<fetcher::Fetcher>>,
    active_tasks: HashMap<Uuid, Task>,
}

pub enum Msg {
    OpenDbUpdate(web_sys::Event),
    OpenDbSuccess(web_sys::Event),
    IdbRequestSuccess(web_sys::Event),
    IdbRequestError(web_sys::Event),
    FetcherMessage(fetcher::Response),
}

pub struct OpenDb {
    _closure_update: Closure<dyn Fn(web_sys::Event)>,
    _closure_success: Closure<dyn Fn(web_sys::Event)>,
    request: web_sys::IdbOpenDbRequest,
}

pub struct IdbReq {
    _closure_error: Closure<dyn Fn(web_sys::Event)>,
    _closure_success: Closure<dyn Fn(web_sys::Event)>,
    request: web_sys::IdbRequest,
}

struct Task {
    request: Request,
    handler_id: HandlerId,
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
                    let task = self.pending_tasks.pop().unwrap();

                    match task.request {
                        Request::GetChannels => {
                            log::info!(
                                "handle input: send channels: handler id: {:?}",
                                task.handler_id
                            );
                            let task_id = Uuid::new_v4();
                            self.active_tasks.insert(task_id, task);
                            self.fetcher.send(fetcher::Request::FetchText(
                                task_id,
                                format!("/api/channels"),
                            ));
                        }
                        Request::AddChannels(channels) => match &self.db {
                            Some(db) => {
                                let trans = db
                                    .transaction_with_str_and_mode(
                                        "channels",
                                        IdbTransactionMode::Readwrite,
                                    )
                                    .unwrap();
                                let os = trans.object_store("channels").unwrap();

                                for channel in channels {
                                    os.put_with_key(
                                        &serde_wasm_bindgen::to_value(&channel).unwrap(),
                                        &serde_wasm_bindgen::to_value(&channel.id).unwrap(),
                                    )
                                    .unwrap();
                                }
                            }
                            None => todo!("implement error handling"),
                        },
                        Request::DownloadEnclosure(id) => {
                            log::info!("requested download of {}", id);

                            let task_id = Uuid::new_v4();
                            self.active_tasks.insert(task_id, task);
                            self.fetcher.send(fetcher::Request::FetchBinary(
                                task_id,
                                format!("/api/items/{}/stream", id),
                            ));
                        }
                        Request::GetEnclosure(id) => {
                            let trans = db
                                .transaction_with_str_and_mode(
                                    "enclosures",
                                    IdbTransactionMode::Readwrite,
                                )
                                .unwrap();
                            let os = trans.object_store("enclosures").unwrap();
                            let req = os.get(&serde_wasm_bindgen::to_value(&id).unwrap()).unwrap();
                            let callback_error = self.link.callback(Msg::IdbRequestError);
                            let callback_success = self.link.callback(Msg::IdbRequestSuccess);
                            let closure_success =
                                Closure::wrap(Box::new(move |event: web_sys::Event| {
                                    callback_success.emit(event)
                                }) as Box<dyn Fn(_)>);
                            req.set_onsuccess(Some(closure_success.as_ref().unchecked_ref()));
                            let closure_error =
                                Closure::wrap(Box::new(move |event: web_sys::Event| {
                                    callback_error.emit(event)
                                }) as Box<dyn Fn(_)>);
                            req.set_onerror(Some(closure_error.as_ref().unchecked_ref()));

                            log::info!(
                                "get enclosure: handler id: {:?} {}",
                                task.handler_id,
                                task.handler_id.is_respondable()
                            );
                            self.idb_request = Some(IdbReq {
                                _closure_error: closure_error,
                                _closure_success: closure_success,
                                request: req,
                            });
                        }
                    }
                }
            }
            None => log::error!("no database available"),
        }
    }
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
            idb_request: None,
            pending_tasks: Vec::new(),
            fetcher: fetcher::Fetcher::bridge(fetcher_cb),
            active_tasks: HashMap::new(),
        };

        obj.init();

        obj
    }

    fn update(&mut self, msg: Self::Message) {
        match msg {
            Msg::FetcherMessage(resp) => match resp {
                fetcher::Response::Binary(uuid, res) => {
                    let task = self.active_tasks.remove(&uuid).unwrap();

                    match task.request {
                        Request::DownloadEnclosure(id) => match (&self.db, res) {
                            (Some(db), Ok(data)) => {
                                let trans = db
                                    .transaction_with_str_and_mode(
                                        "enclosures",
                                        IdbTransactionMode::Readwrite,
                                    )
                                    .unwrap();
                                let os = trans.object_store("enclosures").unwrap();
                                os.put_with_key(&data, &serde_wasm_bindgen::to_value(&id).unwrap())
                                    .unwrap();
                            }
                            (None, _) => log::error!("could not find database"),
                            (_, Err(e)) => log::error!("error downloading enclosure: {}", e),
                        },
                        _ => {}
                    }
                }
                fetcher::Response::Text(uuid, res) => {
                    let task = self.active_tasks.remove(&uuid).unwrap();

                    match task.request {
                        Request::GetChannels => self.link.respond(
                            task.handler_id,
                            Response::Channels(match res {
                                Ok(s) => serde_json::from_str(&s)
                                    .context("conversion to vector of channels failed"),
                                Err(e) => Err(e),
                            }),
                        ),
                        _ => {}
                    }
                }
            },
            Msg::OpenDbUpdate(e) => {
                log::info!("update {:?}", e);
                let idb_db = IdbDatabase::from(
                    self.open_request
                        .as_ref()
                        .unwrap()
                        .request
                        .result()
                        .unwrap(),
                );
                log::info!("db: {:?}", idb_db);
                let idb_object_store = idb_db.create_object_store("channels");
                log::info!("object store: {:?}", idb_object_store);
                match idb_db.create_object_store("enclosures") {
                    Ok(_) => log::info!("created object store \"enclosures\""),
                    Err(e) => log::error!("failed to create object store \"enclosures\": {:?}", e),
                }
            }
            Msg::OpenDbSuccess(e) => {
                log::info!("success {:?}", e);
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
                self.process_tasks();
            }
            Msg::IdbRequestError(e) => {
                log::error!("{:?}", e);
            }
            Msg::IdbRequestSuccess(e) => {
                log::info!("idb request success {:?}", e);
                let req = self.idb_request.as_ref().unwrap();
                let res: ArrayBuffer = req.request.result().unwrap().dyn_into().unwrap();

                for sub in self.subscribers.iter() {
                    self.link
                        .respond(*sub, Response::Enclosure(Ok(res.clone())));
                }
            }
        }
    }

    fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
        log::info!("received task: handler id {:?}", id);

        self.pending_tasks.push(Task {
            request: msg,
            handler_id: id,
        });

        match &self.db {
            Some(_) => self.process_tasks(),
            None => log::warn!("database not available; postponing task"),
        }
    }

    fn connected(&mut self, id: HandlerId) {
        log::info!("connected: handler id: {:?}", id);
        self.subscribers.insert(id);
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
    }
}
