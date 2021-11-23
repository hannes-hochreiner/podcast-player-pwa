use super::fetcher;
use crate::objects::{channel::*, item::*};
use js_sys::ArrayBuffer;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{IdbDatabase, IdbIndexParameters, IdbRequest, IdbTransaction, IdbTransactionMode};
use yew::worker::*;

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    GetChannels,
    GetItems,
    GetItemsByChannelIdYearMonth(Uuid, String),
    AddChannels(Vec<ChannelVal>),
    AddItems(Vec<ItemVal>),
    DownloadEnclosure(Uuid),
    GetEnclosure(Uuid),
    UpdateChannel(Channel),
}

pub enum Response {
    Channels(anyhow::Result<Vec<Channel>>),
    Enclosure(anyhow::Result<ArrayBuffer>),
    AddChannels(anyhow::Result<()>),
    AddItems(anyhow::Result<()>),
    Items(anyhow::Result<Vec<Item>>),
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
    AddChannelValsTask(AddChannelValsTask),
    DownloadEnclosureTask(DownloadEnclosureTask),
    GetEnclosureTask(GetEnclosureTask),
    UpdateChannelTask(UpdateChannelTask),
    AddItemValsTask(AddItemValsTask),
    GetItemsTask(GetItemsTask),
    GetItemsByChannelIdYearMonthTask(GetItemsByChannelIdYearMonthTask),
}

struct UpdateChannelTask {
    channel: Channel,
    handler_id: HandlerId,
}

struct GetChannelsTask {
    channels: Option<Vec<Channel>>,
    transaction: Option<IdbTransaction>,
    handler_id: HandlerId,
}

struct AddChannelValsTask {
    handler_id: HandlerId,
    channel_vals: Vec<ChannelVal>,
    transaction: Option<IdbTransaction>,
    channels: Option<Vec<Channel>>,
}

struct AddItemValsTask {
    handler_id: HandlerId,
    item_vals: Vec<ItemVal>,
    transaction: Option<IdbTransaction>,
    items: Option<Vec<Item>>,
}

struct GetItemsTask {
    items: Option<Vec<Item>>,
    transaction: Option<IdbTransaction>,
    handler_id: HandlerId,
}

struct GetItemsByChannelIdYearMonthTask {
    items: Option<Vec<Item>>,
    transaction: Option<IdbTransaction>,
    handler_id: HandlerId,
    channel_id: Uuid,
    year_month: String,
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
                        InternalTask::UpdateChannelTask(task) => {
                            let transaction = db
                                .transaction_with_str_sequence_and_mode(
                                    &serde_wasm_bindgen::to_value(&vec!["channels"]).unwrap(),
                                    IdbTransactionMode::Readwrite,
                                )
                                .unwrap();
                            let channel_os = transaction.object_store("channels").unwrap();
                            channel_os
                                .put_with_key(
                                    &serde_wasm_bindgen::to_value(&task.channel).unwrap(),
                                    &serde_wasm_bindgen::to_value(&task.channel.val.id).unwrap(),
                                )
                                .unwrap();
                            self.pending_tasks.push(InternalTask::GetChannelsTask(
                                GetChannelsTask {
                                    channels: None,
                                    handler_id: task.handler_id,
                                    transaction: Some(transaction),
                                },
                            ));
                        }
                        InternalTask::GetChannelsTask(mut task) => {
                            if task.transaction.is_none() {
                                task.transaction = Some(
                                    db.transaction_with_str_sequence_and_mode(
                                        &serde_wasm_bindgen::to_value(&vec!["channels"]).unwrap(),
                                        IdbTransactionMode::Readonly,
                                    )
                                    .unwrap(),
                                );
                            }

                            match (&task.channels, &task.transaction) {
                                (None, Some(trans)) => {
                                    let os = trans.object_store("channels").unwrap();
                                    let req = os.get_all().unwrap();

                                    wrap_idb_request(
                                        &self.link,
                                        &mut self.idb_tasks,
                                        InternalTask::GetChannelsTask(task),
                                        req,
                                    );
                                }
                                (Some(channels), Some(_trans)) => self.link.respond(
                                    task.handler_id,
                                    Response::Channels(Ok((*channels).clone())),
                                ),
                                _ => self.link.respond(
                                    task.handler_id,
                                    Response::Channels(Err(anyhow::anyhow!(
                                        "could not get channels"
                                    ))),
                                ),
                            }
                        }
                        InternalTask::GetItemsTask(mut task) => {
                            if task.transaction.is_none() {
                                task.transaction = Some(
                                    db.transaction_with_str_sequence_and_mode(
                                        &serde_wasm_bindgen::to_value(&vec!["items"]).unwrap(),
                                        IdbTransactionMode::Readonly,
                                    )
                                    .unwrap(),
                                );
                            }

                            match (&task.items, &task.transaction) {
                                (None, Some(trans)) => {
                                    let os = trans.object_store("items").unwrap();
                                    let req = os.get_all().unwrap();

                                    wrap_idb_request(
                                        &self.link,
                                        &mut self.idb_tasks,
                                        InternalTask::GetItemsTask(task),
                                        req,
                                    );
                                }
                                (Some(items), Some(_trans)) => self.link.respond(
                                    task.handler_id,
                                    Response::Items(Ok((*items).clone())),
                                ),
                                _ => self.link.respond(
                                    task.handler_id,
                                    Response::Items(Err(anyhow::anyhow!("could not get items"))),
                                ),
                            }
                        }
                        InternalTask::GetItemsByChannelIdYearMonthTask(mut task) => {
                            if task.transaction.is_none() {
                                task.transaction = Some(
                                    db.transaction_with_str_sequence_and_mode(
                                        &serde_wasm_bindgen::to_value(&vec!["items"]).unwrap(),
                                        IdbTransactionMode::Readonly,
                                    )
                                    .unwrap(),
                                );
                            }

                            match (&task.items, &task.transaction) {
                                (None, Some(trans)) => {
                                    let os = trans.object_store("items").unwrap();
                                    let req = os
                                        .index("channel_id_year_month")
                                        .unwrap()
                                        .get_all_with_key(
                                            &serde_wasm_bindgen::to_value(&vec![
                                                task.channel_id.to_string(),
                                                task.year_month.clone(),
                                            ])
                                            .unwrap(),
                                        )
                                        .unwrap();

                                    wrap_idb_request(
                                        &self.link,
                                        &mut self.idb_tasks,
                                        InternalTask::GetItemsByChannelIdYearMonthTask(task),
                                        req,
                                    );
                                }
                                (Some(items), Some(_trans)) => self.link.respond(
                                    task.handler_id,
                                    Response::Items(Ok((*items).clone())),
                                ),
                                _ => self.link.respond(
                                    task.handler_id,
                                    Response::Items(Err(anyhow::anyhow!("could not get items"))),
                                ),
                            }
                        }
                        InternalTask::AddChannelValsTask(mut task) => {
                            if task.transaction.is_none() {
                                task.transaction = Some(
                                    db.transaction_with_str_sequence_and_mode(
                                        &serde_wasm_bindgen::to_value(&vec!["channels"]).unwrap(),
                                        IdbTransactionMode::Readwrite,
                                    )
                                    .unwrap(),
                                );
                            }

                            match (&task.channels, &task.transaction) {
                                (None, Some(trans)) => {
                                    let os = trans.object_store("channels").unwrap();
                                    let req = os.get_all().unwrap();

                                    wrap_idb_request(
                                        &self.link,
                                        &mut self.idb_tasks,
                                        InternalTask::AddChannelValsTask(task),
                                        req,
                                    );
                                }
                                (Some(channels), Some(trans)) => {
                                    let channel_os = trans.object_store("channels").unwrap();
                                    let channel_map: HashMap<Uuid, &Channel> =
                                        channels.iter().map(|e| (e.val.id, e)).collect();

                                    for channel in task.channel_vals {
                                        let channel_new = match channel_map.get(&channel.id) {
                                            Some(&c) => Channel {
                                                val: channel,
                                                meta: c.meta.clone(),
                                            },
                                            None => {
                                                let channel_id = channel.id;

                                                Channel {
                                                    val: channel,
                                                    meta: ChannelMeta {
                                                        id: channel_id,
                                                        active: false,
                                                    },
                                                }
                                            }
                                        };
                                        channel_os
                                            .put_with_key(
                                                &serde_wasm_bindgen::to_value(&channel_new)
                                                    .unwrap(),
                                                &serde_wasm_bindgen::to_value(&channel_new.val.id)
                                                    .unwrap(),
                                            )
                                            .unwrap();
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
                        InternalTask::AddItemValsTask(mut task) => {
                            if task.transaction.is_none() {
                                task.transaction = Some(
                                    db.transaction_with_str_sequence_and_mode(
                                        &serde_wasm_bindgen::to_value(&vec!["items"]).unwrap(),
                                        IdbTransactionMode::Readwrite,
                                    )
                                    .unwrap(),
                                );
                            }

                            match (&task.items, &task.transaction) {
                                (None, Some(trans)) => {
                                    let os = trans.object_store("items").unwrap();
                                    let req = os.get_all().unwrap();

                                    wrap_idb_request(
                                        &self.link,
                                        &mut self.idb_tasks,
                                        InternalTask::AddItemValsTask(task),
                                        req,
                                    );
                                }
                                (Some(items), Some(trans)) => {
                                    let item_os = trans.object_store("items").unwrap();
                                    let item_map: HashMap<Uuid, &Item> =
                                        items.iter().map(|e| (e.val.id, e)).collect();

                                    for item in task.item_vals {
                                        let item_new = match item_map.get(&item.id) {
                                            Some(&i) => Item {
                                                val: item.clone(),
                                                meta: i.meta.clone(),
                                                keys: item.into(),
                                            },
                                            None => {
                                                let item_id = item.id;
                                                Item {
                                                    val: item.clone(),
                                                    meta: ItemMeta {
                                                        download: false,
                                                        id: item_id,
                                                        new: true,
                                                    },
                                                    keys: item.into(),
                                                }
                                            }
                                        };
                                        item_os
                                            .put_with_key(
                                                &serde_wasm_bindgen::to_value(&item_new).unwrap(),
                                                &serde_wasm_bindgen::to_value(&item_new.val.id)
                                                    .unwrap(),
                                            )
                                            .unwrap();
                                    }
                                }
                                _ => self.link.respond(
                                    task.handler_id,
                                    Response::AddItems(Err(anyhow::anyhow!("error adding items"))),
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
                    "items",
                    "enclosures",
                    "enclosures-meta",
                    "images",
                    "images-meta",
                    "configuration",
                ];

                for object_store in object_stores {
                    match idb_db.create_object_store(object_store) {
                        Ok(os) => {
                            log::info!("created object store \"{}\"", object_store);
                            if object_store == "items" {
                                match os.create_index_with_str_sequence_and_optional_parameters(
                                    "channel_id_year_month",
                                    &serde_wasm_bindgen::to_value(&vec![
                                        "val.channel_id",
                                        "keys.year_month",
                                    ])
                                    .unwrap(),
                                    &IdbIndexParameters::new(),
                                ) {
                                    Ok(_) => log::info!("created index"),
                                    Err(e) => log::error!("failed to create index: {:?}", e),
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
                        match &task.channels {
                            None => {
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
                    InternalTask::GetItemsTask(mut task) => {
                        match &task.items {
                            None => {
                                task.items = Some(
                                    serde_wasm_bindgen::from_value(req.request.result().unwrap())
                                        .unwrap(),
                                )
                            }
                            _ => self.link.respond(
                                task.handler_id,
                                Response::Items(Err(anyhow::anyhow!("could not fetch item"))),
                            ),
                        }
                        self.pending_tasks.push(InternalTask::GetItemsTask(task));
                        self.process_tasks();
                    }
                    InternalTask::AddChannelValsTask(mut task) => {
                        match &task.channels {
                            None => {
                                task.channels = Some(
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
                        self.pending_tasks
                            .push(InternalTask::AddChannelValsTask(task));
                        self.process_tasks();
                    }
                    InternalTask::AddItemValsTask(mut task) => {
                        match &task.items {
                            None => {
                                task.items = Some(
                                    serde_wasm_bindgen::from_value(req.request.result().unwrap())
                                        .unwrap(),
                                )
                            }
                            _ => self.link.respond(
                                task.handler_id,
                                Response::AddItems(Err(anyhow::anyhow!(
                                    "could not fetch item meta information"
                                ))),
                            ),
                        }
                        self.pending_tasks.push(InternalTask::AddItemValsTask(task));
                        self.process_tasks();
                    }
                    InternalTask::GetItemsByChannelIdYearMonthTask(mut task) => {
                        match &task.items {
                            None => {
                                task.items = Some(
                                    serde_wasm_bindgen::from_value(req.request.result().unwrap())
                                        .unwrap(),
                                )
                            }
                            _ => self.link.respond(
                                task.handler_id,
                                Response::Items(Err(anyhow::anyhow!("could not fetch item"))),
                            ),
                        }
                        self.pending_tasks
                            .push(InternalTask::GetItemsByChannelIdYearMonthTask(task));
                        self.process_tasks();
                    }
                    _ => {}
                }
            }
        }
    }

    fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
        self.pending_tasks.push(match msg {
            Request::AddChannels(channels) => {
                InternalTask::AddChannelValsTask(AddChannelValsTask {
                    channel_vals: channels,
                    handler_id: id,
                    channels: None,
                    transaction: None,
                })
            }
            Request::AddItems(items) => InternalTask::AddItemValsTask(AddItemValsTask {
                item_vals: items,
                handler_id: id,
                items: None,
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
                channels: None,
                transaction: None,
            }),
            Request::GetEnclosure(uuid) => InternalTask::GetEnclosureTask(GetEnclosureTask {
                handler_id: id,
                uuid,
            }),
            Request::UpdateChannel(channel) => InternalTask::UpdateChannelTask(UpdateChannelTask {
                handler_id: id,
                channel,
            }),
            Request::GetItems => InternalTask::GetItemsTask(GetItemsTask {
                handler_id: id,
                items: None,
                transaction: None,
            }),
            Request::GetItemsByChannelIdYearMonth(channel_id, year_month) => {
                InternalTask::GetItemsByChannelIdYearMonthTask(GetItemsByChannelIdYearMonthTask {
                    channel_id,
                    handler_id: id,
                    items: None,
                    transaction: None,
                    year_month,
                })
            }
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
