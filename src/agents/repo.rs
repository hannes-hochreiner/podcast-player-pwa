use crate::objects::channel::Channel;
use js_sys::ArrayBuffer;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{IdbDatabase, IdbTransactionMode};
use yew::worker::*;
use yewtil::future::LinkFuture;

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    GetChannels,
    DownloadEnclosure(Uuid),
    GetEnclosure(Uuid),
}

pub enum Response {
    Channels(Vec<Channel>),
    Enclosure(ArrayBuffer),
}

pub struct Repo {
    link: AgentLink<Repo>,
    subscribers: HashSet<HandlerId>,
    open_request: Option<OpenDb>,
    db: Option<IdbDatabase>,
    idb_request: Option<IdbReq>,
}

pub enum Msg {
    OpenDbUpdate(web_sys::Event),
    OpenDbSuccess(web_sys::Event),
    IdbRequestSuccess(web_sys::Event),
    IdbRequestError(web_sys::Event),
    ReceiveDownload(Uuid, Result<ArrayBuffer, JsValue>),
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
}

impl Agent for Repo {
    type Reach = Context<Self>;
    type Message = Msg;
    type Input = Request;
    type Output = Response;

    fn create(link: AgentLink<Self>) -> Self {
        let mut obj = Self {
            link,
            subscribers: HashSet::new(),
            open_request: None,
            db: None,
            idb_request: None,
        };

        obj.init();

        obj
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
                log::info!("received data {:?}", data);
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
                            &data.unwrap(),
                            &serde_wasm_bindgen::to_value(&id).unwrap(),
                        )
                        .unwrap();
                    }
                    None => log::error!("could not find database"),
                }
            }
            Msg::IdbRequestError(e) => {
                log::error!("{:?}", e);
            }
            Msg::IdbRequestSuccess(e) => {
                log::info!("idb request success {:?}", e);
                let res: ArrayBuffer = self
                    .idb_request
                    .as_ref()
                    .unwrap()
                    .request
                    .result()
                    .unwrap()
                    .dyn_into()
                    .unwrap();

                for sub in self.subscribers.iter() {
                    self.link.respond(*sub, Response::Enclosure(res.clone()));
                }
            }
        }
    }

    fn handle_input(&mut self, msg: Self::Input, _id: HandlerId) {
        match msg {
            Request::GetChannels => {
                for sub in self.subscribers.iter() {
                    // self.link.respond(*sub, s.clone());
                }
            }
            Request::DownloadEnclosure(id) => {
                log::info!("requested download of {}", id);

                self.link.send_future(async move {
                    Msg::ReceiveDownload(
                        id,
                        fetch_binary(&format!("/api/items/{}/stream", id)).await,
                    )
                });
            }
            Request::GetEnclosure(id) => {
                log::info!("get enclosure db {:?}", self.db);
                let trans = self
                    .db
                    .as_ref()
                    .unwrap()
                    .transaction_with_str_and_mode("enclosures", IdbTransactionMode::Readwrite)
                    .unwrap();
                let os = trans.object_store("enclosures").unwrap();
                let req = os.get(&serde_wasm_bindgen::to_value(&id).unwrap()).unwrap();
                let callback_error = self.link.callback(Msg::IdbRequestError);
                let callback_success = self.link.callback(Msg::IdbRequestSuccess);
                let closure_success = Closure::wrap(Box::new(move |event: web_sys::Event| {
                    callback_success.emit(event)
                }) as Box<dyn Fn(_)>);
                req.set_onsuccess(Some(closure_success.as_ref().unchecked_ref()));
                let closure_error =
                    Closure::wrap(
                        Box::new(move |event: web_sys::Event| callback_error.emit(event))
                            as Box<dyn Fn(_)>,
                    );
                req.set_onerror(Some(closure_error.as_ref().unchecked_ref()));

                self.idb_request = Some(IdbReq {
                    _closure_error: closure_error,
                    _closure_success: closure_success,
                    request: req,
                });
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

// https://github.com/yewstack/yew/blob/v0.18/examples/futures/src/main.rs
async fn fetch_binary(url: &str) -> Result<ArrayBuffer, wasm_bindgen::JsValue> {
    let mut opts = web_sys::RequestInit::new();
    opts.method("GET");
    // opts.mode(web_sys::RequestMode::Cors);

    let request = web_sys::Request::new_with_str_and_init(url, &opts)?;

    let window = yew::utils::window();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: web_sys::Response = resp_value.dyn_into().unwrap();

    log::info!("fetch enclosure: {:?}", resp);

    let buffer = JsFuture::from(resp.array_buffer()?).await?;

    log::info!("buffer: {:?}", buffer);
    Ok(ArrayBuffer::from(buffer))
}
