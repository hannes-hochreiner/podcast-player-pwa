use super::fetcher;
use crate::objects::{
    channel::{self, Channel},
    channel_meta::{self, ChannelMeta},
};
use js_sys::ArrayBuffer;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{IdbDatabase, IdbRequest, IdbTransaction, IdbTransactionMode};
use yew::worker::*;

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    GetChannels,
    AddChannels(Vec<Channel>),
    DownloadEnclosure(Uuid),
    GetEnclosure(Uuid),
}

pub enum Response {
    Channels(anyhow::Result<(Vec<Channel>, Vec<ChannelMeta>)>),
    Enclosure(anyhow::Result<ArrayBuffer>),
    AddChannels(anyhow::Result<()>),
}

pub struct Repo {
    link: AgentLink<Repo>,
    subscribers: HashSet<HandlerId>,
    open_request: Option<OpenDb>,
    db: Option<IdbDatabase>,
    pending_tasks: Vec<InternalTask>,
    fetcher: Box<dyn Bridge<fetcher::Fetcher>>,
    idb_tasks: HashMap<Uuid, IdbResponse>,
    fetcher_tasks: HashMap<Uuid, InternalTask>,
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
    task: InternalTask,
}

enum InternalTask {
    GetChannelsTask(GetChannelsTask),
    AddChannelsTask(AddChannelsTask),
    DownloadEnclosureTask(DownloadEnclosureTask),
    GetEnclosureTask(GetEnclosureTask),
}

struct GetChannelsTask {
    metas: Option<Vec<channel_meta::ChannelMeta>>,
    channels: Option<Vec<channel::Channel>>,
    transaction: Option<IdbTransaction>,
    handler_id: HandlerId,
}

struct AddChannelsTask {
    handler_id: HandlerId,
    metas: Option<Vec<channel_meta::ChannelMeta>>,
    transaction: Option<IdbTransaction>,
    channels: Vec<Channel>,
}

struct DownloadEnclosureTask {
    // handler_id: HandlerId,
    uuid: Uuid,
}

struct GetEnclosureTask {
    handler_id: HandlerId,
    uuid: Uuid,
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

                    match task {
                        InternalTask::GetChannelsTask(mut task) => {
                            if task.transaction.is_none() {
                                task.transaction = Some(
                                    db.transaction_with_str_sequence_and_mode(
                                        &serde_wasm_bindgen::to_value(&vec![
                                            "channels",
                                            "channels-meta",
                                        ])
                                        .unwrap(),
                                        IdbTransactionMode::Readonly,
                                    )
                                    .unwrap(),
                                );
                            }

                            match (&task.metas, &task.channels, &task.transaction) {
                                (None, None, Some(trans)) => {
                                    let os = trans.object_store("channels-meta").unwrap();
                                    let req = os.get_all().unwrap();

                                    wrap_idb_request(
                                        &self.link,
                                        &mut self.idb_tasks,
                                        InternalTask::GetChannelsTask(task),
                                        req,
                                    );
                                }
                                (Some(_), None, Some(trans)) => {
                                    let os = trans.object_store("channels").unwrap();
                                    let req = os.get_all().unwrap();

                                    wrap_idb_request(
                                        &self.link,
                                        &mut self.idb_tasks,
                                        InternalTask::GetChannelsTask(task),
                                        req,
                                    );
                                }
                                (Some(metas), Some(channels), _) => self.link.respond(
                                    task.handler_id,
                                    Response::Channels(Ok(((*channels).clone(), (*metas).clone()))),
                                ),
                                _ => self.link.respond(
                                    task.handler_id,
                                    Response::Channels(Err(anyhow::anyhow!(
                                        "could not get channels"
                                    ))),
                                ),
                            }
                        }
                        InternalTask::AddChannelsTask(mut task) => {
                            if task.transaction.is_none() {
                                task.transaction = Some(
                                    db.transaction_with_str_sequence_and_mode(
                                        &serde_wasm_bindgen::to_value(&vec![
                                            "channels",
                                            "channels-meta",
                                        ])
                                        .unwrap(),
                                        IdbTransactionMode::Readwrite,
                                    )
                                    .unwrap(),
                                );
                            }

                            match (&task.metas, &task.transaction) {
                                (None, Some(trans)) => {
                                    let os = trans.object_store("channels-meta").unwrap();
                                    let req = os.get_all().unwrap();

                                    wrap_idb_request(
                                        &self.link,
                                        &mut self.idb_tasks,
                                        InternalTask::AddChannelsTask(task),
                                        req,
                                    );
                                }
                                (Some(metas), Some(trans)) => {
                                    let channel_os = trans.object_store("channels").unwrap();
                                    let channel_meta_os =
                                        trans.object_store("channels-meta").unwrap();
                                    let metas: Vec<Uuid> = metas.iter().map(|e| e.id).collect();

                                    for channel in task.channels {
                                        channel_os
                                            .put_with_key(
                                                &serde_wasm_bindgen::to_value(&channel).unwrap(),
                                                &serde_wasm_bindgen::to_value(&channel.id).unwrap(),
                                            )
                                            .unwrap();
                                        if !metas.contains(&channel.id) {
                                            channel_meta_os
                                                .put_with_key(
                                                    &serde_wasm_bindgen::to_value(&ChannelMeta {
                                                        active: false,
                                                        id: channel.id,
                                                    })
                                                    .unwrap(),
                                                    &serde_wasm_bindgen::to_value(&channel.id)
                                                        .unwrap(),
                                                )
                                                .unwrap();
                                        }
                                    }
                                }
                                _ => self.link.respond(
                                    task.handler_id,
                                    Response::AddChannels(Err(anyhow::anyhow!(
                                        "error adding channels"
                                    ))),
                                ),
                            }
                        }
                        InternalTask::DownloadEnclosureTask(task) => {
                            let task_id = Uuid::new_v4();
                            self.fetcher.send(fetcher::Request::FetchBinary(
                                task_id,
                                format!("/api/items/{}/stream", task.uuid),
                            ));
                            self.fetcher_tasks
                                .insert(task_id, InternalTask::DownloadEnclosureTask(task));
                        }
                        InternalTask::GetEnclosureTask(task) => {
                            let trans = db
                                .transaction_with_str_and_mode(
                                    "enclosures",
                                    IdbTransactionMode::Readwrite,
                                )
                                .unwrap();
                            let os = trans.object_store("enclosures").unwrap();
                            let req = os
                                .get(&serde_wasm_bindgen::to_value(&task.uuid).unwrap())
                                .unwrap();

                            wrap_idb_request(
                                &self.link,
                                &mut self.idb_tasks,
                                InternalTask::GetEnclosureTask(task),
                                req,
                            );
                        }
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
    task: InternalTask,
    request: IdbRequest,
) {
    let task_id = Uuid::new_v4();
    let callback_success = link.callback(Msg::IdbRequest);
    let callback_error = link.callback(Msg::IdbRequest);
    let closure_success = Closure::wrap(Box::new(move |event: web_sys::Event| {
        callback_success.emit((task_id, Ok(event)))
    }) as Box<dyn Fn(_)>);
    let closure_error = Closure::wrap(Box::new(move |event: web_sys::Event| {
        callback_error.emit((task_id, Err(event)))
    }) as Box<dyn Fn(_)>);
    request.set_onsuccess(Some(closure_error.as_ref().unchecked_ref()));
    request.set_onerror(Some(closure_error.as_ref().unchecked_ref()));
    active_idb_tasks.insert(
        task_id,
        IdbResponse {
            request,
            _closure_error: closure_error,
            _closure_success: closure_success,
            task,
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
                        InternalTask::DownloadEnclosureTask(task) => match (&self.db, res) {
                            (Some(db), Ok(data)) => {
                                let trans = db
                                    .transaction_with_str_and_mode(
                                        "enclosures",
                                        IdbTransactionMode::Readwrite,
                                    )
                                    .unwrap();
                                let os = trans.object_store("enclosures").unwrap();
                                os.put_with_key(
                                    &data,
                                    &serde_wasm_bindgen::to_value(&task.uuid).unwrap(),
                                )
                                .unwrap();
                            }
                            (None, _) => log::error!("could not find database"),
                            (_, Err(e)) => log::error!("error downloading enclosure: {}", e),
                        },
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
                    "channels-meta",
                    "items",
                    "items-meta",
                    "enclosures",
                    "enclosures-meta",
                    "images",
                    "images-meta",
                ];

                for object_store in object_stores {
                    match idb_db.create_object_store(object_store) {
                        Ok(_) => log::info!("created object store \"{}\"", object_store),
                        Err(e) => {
                            log::error!(
                                "failed to create object store \"{}\": {:?}",
                                object_store,
                                e
                            )
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
                self.process_tasks();
            }
            Msg::IdbRequest(res) => {
                let req = self.idb_tasks.remove(&res.0).unwrap();

                match req.task {
                    InternalTask::GetEnclosureTask(task) => {
                        let res: ArrayBuffer = req.request.result().unwrap().dyn_into().unwrap();

                        self.link
                            .respond(task.handler_id, Response::Enclosure(Ok(res)));
                    }
                    InternalTask::GetChannelsTask(mut task) => {
                        match (&task.metas, &task.channels) {
                            (None, None) => {
                                task.metas = Some(
                                    serde_wasm_bindgen::from_value(req.request.result().unwrap())
                                        .unwrap(),
                                )
                            }
                            (Some(_), None) => {
                                task.channels = Some(
                                    serde_wasm_bindgen::from_value(req.request.result().unwrap())
                                        .unwrap(),
                                )
                            }
                            _ => self.link.respond(
                                task.handler_id,
                                Response::Channels(Err(anyhow::anyhow!(
                                    "could not fetch channels"
                                ))),
                            ),
                        }
                        self.pending_tasks.push(InternalTask::GetChannelsTask(task));
                        self.process_tasks();
                    }
                    InternalTask::AddChannelsTask(mut task) => {
                        match &task.metas {
                            None => {
                                task.metas = Some(
                                    serde_wasm_bindgen::from_value(req.request.result().unwrap())
                                        .unwrap(),
                                )
                            }
                            _ => self.link.respond(
                                task.handler_id,
                                Response::AddChannels(Err(anyhow::anyhow!(
                                    "could not fetch channel meta information"
                                ))),
                            ),
                        }
                        self.pending_tasks.push(InternalTask::AddChannelsTask(task));
                        self.process_tasks();
                    }
                    _ => {}
                }
            }
        }
    }

    fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
        self.pending_tasks.push(match msg {
            Request::AddChannels(channels) => InternalTask::AddChannelsTask(AddChannelsTask {
                channels,
                handler_id: id,
                metas: None,
                transaction: None,
            }),
            Request::DownloadEnclosure(uuid) => {
                InternalTask::DownloadEnclosureTask(DownloadEnclosureTask {
                    // handler_id: id,
                    uuid,
                })
            }
            Request::GetChannels => InternalTask::GetChannelsTask(GetChannelsTask {
                handler_id: id,
                metas: None,
                transaction: None,
                channels: None,
            }),
            Request::GetEnclosure(uuid) => InternalTask::GetEnclosureTask(GetEnclosureTask {
                handler_id: id,
                uuid,
            }),
        });

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
