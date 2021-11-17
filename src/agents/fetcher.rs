use anyhow::Result;
use js_sys::ArrayBuffer;
use std::collections::HashSet;
use uuid::Uuid;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use yew::worker::*;
use yewtil::future::LinkFuture;

pub enum Request {
    FetchText(Uuid, String),
    FetchBinary(Uuid, String),
}

pub enum Response {
    Binary(Uuid, Result<ArrayBuffer>),
    Text(Uuid, Result<String>),
}

pub enum Message {
    ReceiveText(HandlerId, Uuid, Result<String>),
    ReceiveBinary(HandlerId, Uuid, Result<ArrayBuffer>),
}

pub struct Fetcher {
    link: AgentLink<Self>,
    subscribers: HashSet<HandlerId>,
}

impl Agent for Fetcher {
    type Reach = Context<Self>;
    type Message = Message;
    type Input = Request;
    type Output = Response;

    fn create(link: AgentLink<Self>) -> Self {
        Self {
            link,
            subscribers: HashSet::<HandlerId>::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) {
        match msg {
            Message::ReceiveBinary(handler_id, uuid, res) => {
                self.link.respond(handler_id, Response::Binary(uuid, res))
            }
            Message::ReceiveText(handler_id, uuid, res) => {
                self.link.respond(handler_id, Response::Text(uuid, res))
            }
        }
    }

    fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
        match msg {
            Request::FetchBinary(uuid, url) => {
                self.link.send_future(async move {
                    Message::ReceiveBinary(id, uuid, fetch_binary(&url).await)
                });
            }
            Request::FetchText(uuid, url) => {
                self.link.send_future(async move {
                    Message::ReceiveText(id, uuid, fetch_text(&url).await)
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

async fn fetch(url: &str) -> Result<web_sys::Response, wasm_bindgen::JsValue> {
    let mut opts = web_sys::RequestInit::new();
    opts.method("GET");

    let request = web_sys::Request::new_with_str_and_init(url, &opts)?;

    let window = yew::utils::window();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: web_sys::Response = resp_value.dyn_into().unwrap();

    log::info!("fetch enclosure: {:?}", resp);

    Ok(resp)
}

// https://github.com/yewstack/yew/blob/v0.18/examples/futures/src/main.rs
async fn fetch_binary(url: &str) -> Result<ArrayBuffer> {
    Ok(ArrayBuffer::from(
        JsFuture::from(
            fetch(url)
                .await
                .map_err(|_| anyhow::anyhow!("fetch failed"))?
                .array_buffer()
                .map_err(|_| anyhow::anyhow!("retrieving arraybuffer failed"))?,
        )
        .await
        .map_err(|_| anyhow::anyhow!("creating future failed"))?,
    ))
}

async fn fetch_text(url: &str) -> Result<String> {
    JsFuture::from(
        fetch(url)
            .await
            .map_err(|_| anyhow::anyhow!("fetch failed"))?
            .text()
            .map_err(|_| anyhow::anyhow!("retrieving text failed"))?,
    )
    .await
    .map_err(|_| anyhow::anyhow!("creating future failed"))?
    .as_string()
    .ok_or(anyhow::anyhow!("could not convert response into string"))
}
