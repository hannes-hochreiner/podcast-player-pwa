use crate::objects::channel::Channel;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{IdbDatabase, IdbTransactionMode};
use yew::format::Nothing;
use yew::services::fetch;
use yew::worker::*;

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    GetChannels,
    DownloadEnclosure(Uuid),
}

pub enum Response {
    Channels(Vec<Channel>),
}

pub struct Repo {
    link: AgentLink<Repo>,
    subscribers: HashSet<HandlerId>,
    open_request: Option<OpenDb>,
    download_task: Option<fetch::FetchTask>,
    db: Option<IdbDatabase>,
}

pub enum Msg {
    OpenDbUpdate(web_sys::Event),
    OpenDbSuccess(web_sys::Event),
    ReceiveDownload(Uuid, Vec<u8>),
}

pub struct OpenDb {
    _closure_update: Closure<dyn Fn(web_sys::Event)>,
    _closure_success: Closure<dyn Fn(web_sys::Event)>,
    request: web_sys::IdbOpenDbRequest,
}

impl Repo {
    fn start_task(&mut self, req: Request) {
        let window: web_sys::Window = web_sys::window().expect("window not available");
        let idb_factory: web_sys::IdbFactory = window.indexed_db().unwrap().unwrap();
        // let mut idb_options = web_sys::IdbOpenDbOptions::new();

        // idb_options.version(1f64);
        // idb_options.storage(web_sys::StorageType::Persistent);
        // let idb_open_request: web_sys::IdbOpenDbRequest = idb_factory.open_with_idb_open_db_options("more-podcasts", &idb_options).unwrap();

        let idb_open_request: web_sys::IdbOpenDbRequest =
            idb_factory.open_with_u32("podcast-player", 1).unwrap();
        let callback_update = self.link.callback(Msg::OpenDbUpdate);
        let callback_success = self.link.callback(Msg::OpenDbSuccess);
        // let callback = wasm_bindgen::closure::Closure::wrap(Box::new(move |e| {
        //     &self.callback(e);
        // }) as Box<dyn FnMut(web_sys::Event)>);

        // idb_open_request.set_onupgradeneeded(Some(callback.as_ref().unchecked_ref()));

        // idb_open_request.set_onupgradeneeded(Some(wasm_bindgen::closure::Closure::once_into_js(move |event: web_sys::Event| {
        //     callback.emit(event)
        // }).unchecked_ref()));

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

        // let
        // window.alert_with_message("hello from wasm!").expect("alert failed");
    }
}

impl Agent for Repo {
    type Reach = Context<Self>;
    type Message = Msg;
    type Input = Request;
    type Output = Response;

    fn create(link: AgentLink<Self>) -> Self {
        Self {
            link,
            subscribers: HashSet::new(),
            open_request: None,
            download_task: None,
            db: None,
        }
    }

    fn update(&mut self, msg: Self::Message) {
        match msg {
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
            }
            Msg::ReceiveDownload(id, data) => {
                log::info!("received data {}", data.len());
                log::info!("{:?}", self.db);
                match &self.db {
                    Some(db) => {
                        let trans = db
                            .transaction_with_str_and_mode(
                                "enclosures",
                                IdbTransactionMode::Readwrite,
                            )
                            .unwrap();
                        let os = trans.object_store("enclosures").unwrap();
                        os.put_with_key(
                            &serde_wasm_bindgen::to_value(&data).unwrap(),
                            &serde_wasm_bindgen::to_value(&id).unwrap(),
                        )
                        .unwrap();
                    }
                    None => log::error!("could not find database"),
                }
            }
        }
    }

    fn handle_input(&mut self, msg: Self::Input, _id: HandlerId) {
        match msg {
            Request::GetChannels => {
                log::info!("starting task");
                self.start_task(msg);
                for sub in self.subscribers.iter() {
                    // self.link.respond(*sub, s.clone());
                }
            }
            Request::DownloadEnclosure(id) => {
                log::info!("requested download of {}", id);
                let request = fetch::Request::get(format!("/api/items/{}/stream", id))
                    .body(Nothing)
                    .expect("Could not build request.");
                let callback = self.link.callback(
                    move |response: fetch::Response<Result<Vec<u8>, anyhow::Error>>| {
                        Msg::ReceiveDownload(id, response.into_body().unwrap())
                    },
                );
                let task = fetch::FetchService::fetch_binary(request, callback)
                    .expect("failed to start request");

                self.download_task = Some(task);
            }
        }
    }

    fn connected(&mut self, id: HandlerId) {
        self.subscribers.insert(id);
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
    }
}
