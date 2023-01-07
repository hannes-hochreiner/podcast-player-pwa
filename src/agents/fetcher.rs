use super::notifier;
use crate::{objects::JsError, utils};
use chrono::{DateTime, FixedOffset};
use js_sys::ArrayBuffer;
use podcast_player_common::{channel_val::ChannelVal, item_val::ItemVal, FeedVal};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::ConnectionType;
use yew_agent::{Dispatched, Dispatcher, HandlerId, Public, Worker, WorkerLink};

#[derive(Debug)]
pub enum Request {
    FetchText(Uuid, String),
    // FetchBinary(Uuid, String),
    // PostString(Uuid, String, String),
    PullFeedVals(Option<DateTime<FixedOffset>>),
    PullChannelVals(Option<DateTime<FixedOffset>>),
    PullItemVals(Option<DateTime<FixedOffset>>),
    PullDownload(Uuid),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    Binary(Uuid, Result<ArrayBuffer, JsError>),
    Text(Uuid, Result<String, JsError>),
    PullFeedVals(Result<Vec<FeedVal>, JsError>),
    PullChannelVals(Result<Vec<ChannelVal>, JsError>),
    PullItemVals(Result<Vec<ItemVal>, JsError>),
    PullDownload(Uuid, Result<ArrayBuffer, JsError>),
    PullDownloadStarted(Uuid),
}

#[derive(Debug)]
pub enum Message {
    ReceiveText(HandlerId, Uuid, Result<String, JsError>),
    // ReceiveBinary(HandlerId, Uuid, Result<ArrayBuffer, JsError>),
    PullFeedVals(HandlerId, Result<Vec<FeedVal>, JsError>),
    PullChannelVals(HandlerId, Result<Vec<ChannelVal>, JsError>),
    PullItemVals(HandlerId, Result<Vec<ItemVal>, JsError>),
    PullDownload(HandlerId, Uuid, Result<ArrayBuffer, JsError>),
}

pub struct Fetcher {
    link: WorkerLink<Self>,
    subscribers: HashSet<HandlerId>,
    notifier: Dispatcher<notifier::Notifier>,
}

enum HttpMethod {
    Get,
    Post,
}

impl Fetcher {
    fn process_update(&mut self, msg: Message) -> Result<(), JsError> {
        match msg {
            Message::PullFeedVals(handler_id, res) => {
                self.link.respond(handler_id, Response::PullFeedVals(res));
            }
            Message::PullChannelVals(handler_id, res) => {
                self.link
                    .respond(handler_id, Response::PullChannelVals(res));
            }
            Message::PullItemVals(handler_id, res) => {
                self.link.respond(handler_id, Response::PullItemVals(res));
            }
            Message::PullDownload(handler_id, item_id, res) => self
                .link
                .respond(handler_id, Response::PullDownload(item_id, res)),
            // Message::ReceiveBinary(handler_id, uuid, res) => {
            //     self.link.respond(handler_id, Response::Binary(uuid, res));
            // }
            Message::ReceiveText(handler_id, uuid, res) => {
                self.link.respond(handler_id, Response::Text(uuid, res));
            }
        }

        Ok(())
    }

    fn process_handle_input(&mut self, msg: Request, id: HandlerId) -> Result<(), JsError> {
        let conn_type = utils::get_connection_type()?;

        if (conn_type != ConnectionType::Ethernet)
            & (conn_type != ConnectionType::Wifi)
            & (conn_type != ConnectionType::Unknown)
        {
            return Ok(());
        }

        match msg {
            Request::PullFeedVals(since) => {
                let base_url = "/api/feeds".to_string();
                let url = match since {
                    Some(date) => format!(
                        "{}?{}",
                        base_url,
                        encode_query_pairs(&vec![("since", &date.to_rfc3339())])
                    ),
                    None => base_url,
                };

                self.link.send_future(async move {
                    Message::PullFeedVals(
                        id,
                        fetch_deserializable(&url, HttpMethod::Get, None, None).await,
                    )
                });
            }
            Request::PullChannelVals(since) => {
                let base_url = "/api/channels".to_string();
                let url = match since {
                    Some(date) => format!(
                        "{}?{}",
                        base_url,
                        encode_query_pairs(&vec![("since", &date.to_rfc3339())])
                    ),
                    None => base_url,
                };

                self.link.send_future(async move {
                    Message::PullChannelVals(
                        id,
                        fetch_deserializable(&url, HttpMethod::Get, None, None).await,
                    )
                });
            }
            Request::PullItemVals(since) => {
                let base_url = "/api/items".to_string();
                let url = match since {
                    Some(date) => format!(
                        "{}?{}",
                        base_url,
                        encode_query_pairs(&vec![("since", &date.to_rfc3339())])
                    ),
                    None => base_url,
                };

                self.link.send_future(async move {
                    Message::PullItemVals(
                        id,
                        fetch_deserializable(&url, HttpMethod::Get, None, None).await,
                    )
                });
            }
            Request::PullDownload(item_id) => {
                let url = format!("/api/items/{}/stream", item_id);

                self.link.send_future(async move {
                    Message::PullDownload(id, item_id, fetch_binary(&url, None).await)
                });
                self.link
                    .respond(id, Response::PullDownloadStarted(item_id));
            }
            // Request::FetchBinary(uuid, url) => {
            //     self.link.send_future(async move {
            //         Message::ReceiveBinary(id, uuid, fetch_binary(&url, None).await)
            //     });
            // }
            Request::FetchText(uuid, url) => {
                self.link.send_future(async move {
                    Message::ReceiveText(
                        id,
                        uuid,
                        fetch_text(&url, HttpMethod::Get, None, None).await,
                    )
                });
            } // Request::PostString(uuid, url, body) => {
              //     self.link.send_future(async move {
              //         Message::ReceiveText(
              //             id,
              //             uuid,
              //             fetch_text(&url, HttpMethod::Post, None, Some(body)).await,
              //         )
              //     });
              // }
        }

        Ok(())
    }
}

impl Worker for Fetcher {
    type Reach = Public<Self>;
    type Message = Message;
    type Input = Request;
    type Output = Response;

    fn create(link: WorkerLink<Self>) -> Self {
        Self {
            link,
            subscribers: HashSet::<HandlerId>::new(),
            notifier: notifier::Notifier::dispatcher(),
        }
    }

    fn update(&mut self, msg: Self::Message) {
        // log::debug!("fetcher: update: {:?}", msg);
        match self.process_update(msg) {
            Ok(_) => {}
            Err(e) => self.notifier.send(notifier::Request::NotifyError(e)),
        }
    }

    fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
        // log::debug!("fetcher: handle_input: {:?}", msg);
        match self.process_handle_input(msg, id) {
            Ok(_) => {}
            Err(e) => self.notifier.send(notifier::Request::NotifyError(e)),
        }
    }

    fn connected(&mut self, id: HandlerId) {
        self.subscribers.insert(id);
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
    }
}

fn encode_query_pairs(pairs: &Vec<(&str, &str)>) -> String {
    let mut tmp = url::form_urlencoded::Serializer::new(String::new());

    for (key, value) in pairs {
        tmp.append_pair(key, value);
    }

    tmp.finish()
}

async fn fetch(
    url: &str,
    method: HttpMethod,
    headers: Option<HashMap<String, String>>,
    body: Option<String>,
) -> Result<web_sys::Response, JsError> {
    let mut opts = web_sys::RequestInit::new();

    match method {
        HttpMethod::Get => opts.method("GET"),
        HttpMethod::Post => opts.method("POST"),
    };

    if let Some(headers) = headers {
        let opt_headers = web_sys::Headers::new()?;

        for (key, val) in headers {
            opt_headers.append(&key, &val)?;
        }

        opts.headers(&opt_headers);
    }

    if let Some(val) = body {
        opts.body(Some(&serde_wasm_bindgen::to_value(&val)?));
    }

    let request = web_sys::Request::new_with_str_and_init(url, &opts)?;
    let window = web_sys::window().ok_or("error getting window")?;
    let resp: web_sys::Response = JsFuture::from(window.fetch_with_request(&request))
        .await?
        .dyn_into()?;

    match resp.ok() {
        true => Ok(resp),
        false => {
            Err((&*format!("fetcher error: {}: {}", resp.status(), resp.status_text())).into())
        }
    }
}

// https://github.com/yewstack/yew/blob/v0.18/examples/futures/src/main.rs
async fn fetch_binary(
    url: &str,
    headers: Option<HashMap<String, String>>,
) -> Result<ArrayBuffer, JsError> {
    Ok(ArrayBuffer::from(
        JsFuture::from(
            fetch(url, HttpMethod::Get, headers, None)
                .await?
                .array_buffer()?,
        )
        .await?,
    ))
}

async fn fetch_text(
    url: &str,
    method: HttpMethod,
    headers: Option<HashMap<String, String>>,
    body: Option<String>,
) -> Result<String, JsError> {
    JsFuture::from(fetch(url, method, headers, body).await?.text()?)
        .await?
        .as_string()
        .ok_or("error casting fetched value to string".into())
}

async fn fetch_deserializable<T: DeserializeOwned>(
    url: &str,
    method: HttpMethod,
    headers: Option<HashMap<String, String>>,
    body: Option<String>,
) -> Result<T, JsError> {
    JsFuture::from(fetch(url, method, headers, body).await?.json()?)
        .await
        .map(|val| serde_wasm_bindgen::from_value(val).map_err(Into::into))?
}
