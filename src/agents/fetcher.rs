use crate::{
    objects::{
        Auth0Token, Authorization, AuthorizationConfig, AuthorizationTask, FetcherConfig, JsError,
    },
    utils,
};

use super::{notifier, repo};
use chrono::{DateTime, Duration, FixedOffset, Utc};
use js_sys::ArrayBuffer;
use podcast_player_common::{channel_val::ChannelVal, item_val::ItemVal, FeedVal};
use serde::de::DeserializeOwned;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use url::Url;
use uuid::Uuid;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::ConnectionType;
use yew_agent::{Agent, AgentLink, Bridge, Bridged, Context, Dispatched, Dispatcher, HandlerId};

#[derive(Debug)]
pub enum Request {
    FetchText(Uuid, String),
    FetchBinary(Uuid, String),
    PostString(Uuid, String, String),
    PullFeedVals(Option<DateTime<FixedOffset>>),
    PullChannelVals(Option<DateTime<FixedOffset>>),
    PullItemVals(Option<DateTime<FixedOffset>>),
    PullDownload(Uuid),
}

#[derive(Debug)]
pub enum Response {
    Binary(Uuid, Result<ArrayBuffer, JsError>),
    Text(Uuid, Result<String, JsError>),
    PullFeedVals(Vec<FeedVal>),
    PullChannelVals(Vec<ChannelVal>),
    PullItemVals(Vec<ItemVal>),
    PullDownload(Uuid, ArrayBuffer),
    PullDownloadStarted(Uuid),
}

#[derive(Debug)]
pub enum Message {
    ReceiveText(HandlerId, Uuid, Result<String, JsError>),
    ReceiveBinary(HandlerId, Uuid, Result<ArrayBuffer, JsError>),
    GetToken(Result<Auth0Token, JsError>),
    RepoMessage(repo::Response),
    GetConfig(Result<AuthorizationConfig, JsError>),
    PullFeedVals(HandlerId, Result<Vec<FeedVal>, JsError>),
    PullChannelVals(HandlerId, Result<Vec<ChannelVal>, JsError>),
    PullItemVals(HandlerId, Result<Vec<ItemVal>, JsError>),
    PullDownload(HandlerId, Uuid, Result<ArrayBuffer, JsError>),
}

pub struct Fetcher {
    link: AgentLink<Self>,
    subscribers: HashSet<HandlerId>,
    repo: Box<dyn Bridge<repo::Repo>>,
    config: Option<FetcherConfig>,
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
                self.link.respond(handler_id, Response::PullFeedVals(res?));
            }
            Message::PullChannelVals(handler_id, res) => {
                self.link
                    .respond(handler_id, Response::PullChannelVals(res?));
            }
            Message::PullItemVals(handler_id, res) => {
                self.link.respond(handler_id, Response::PullItemVals(res?));
            }
            Message::PullDownload(handler_id, item_id, res) => self
                .link
                .respond(handler_id, Response::PullDownload(item_id, res?)),
            Message::ReceiveBinary(handler_id, uuid, res) => {
                self.link.respond(handler_id, Response::Binary(uuid, res));
            }
            Message::ReceiveText(handler_id, uuid, res) => {
                self.link.respond(handler_id, Response::Text(uuid, res));
            }
            Message::GetToken(res) => {
                let mut config = self.config.as_ref().ok_or("configuration not set")?.clone();
                let token = res?;

                config.authorization_task = None;
                config.authorization = Some(Authorization {
                    access_token: token.access_token,
                    token_type: token.token_type,
                    expires_at: (Utc::now() + Duration::seconds(token.expires_in - 10)).into(),
                });
                self.repo.send(repo::Request::GetFetcherConf(Some(config)));
            }
            Message::GetConfig(res) => {
                let conf = res?;

                self.repo.send(repo::Request::GetFetcherConf(Some(
                    FetcherConfig::new_with_config(conf),
                )));
            }
            Message::RepoMessage(repo_msg) => match repo_msg {
                repo::Response::FetcherConfig(fetcher_config) => match fetcher_config {
                    Some(fc) => {
                        self.config = Some(fc.clone());

                        match fc.authorization {
                            // unwrap is safe, as we set the configuration two lines earlier
                            None => match &self.config.as_mut().unwrap().authorization_task {
                                Some(at) => {
                                    let url = get_url()?;

                                    match url.query_pairs().find(|(key, _)| key == "code") {
                                        Some((_, val)) => {
                                            let token_url =
                                                format!("{}/oauth/token", fc.config.domain);
                                            let body = format!("grant_type=authorization_code&client_id={}&code_verifier={}&code={}&redirect_uri={}", fc.config.client_id, at.verifier, val, at.redirect);
                                            let mut headers: HashMap<String, String> =
                                                HashMap::new();
                                            headers.insert(
                                                "Content-Type".into(),
                                                "application/x-www-form-urlencoded".into(),
                                            );

                                            self.link.send_future(async move {
                                                Message::GetToken(
                                                    fetch_deserializable(
                                                        &token_url,
                                                        HttpMethod::Post,
                                                        Some(headers),
                                                        Some(body),
                                                    )
                                                    .await,
                                                )
                                            });
                                        }
                                        None => {
                                            let mut url = Url::parse(&format!(
                                                "{}/authorize",
                                                fc.config.domain
                                            ))?;
                                            url.query_pairs_mut()
                                                            .append_pair("audience", &fc.config.audience)
                                                            .append_pair(
                                                                "scope",
                                                                "openid profile read:channels read:items read:feeds write:feeds",
                                                            )
                                                            .append_pair("response_type", "code")
                                                            .append_pair("client_id", &fc.config.client_id)
                                                            .append_pair("redirect_uri", &at.redirect)
                                                            .append_pair("code_challenge", &at.challenge)
                                                            .append_pair("code_challenge_method", "S256");

                                            web_sys::window()
                                                .ok_or("could not get window")?
                                                .location()
                                                .set_href(url.as_str())?;
                                        }
                                    }
                                }
                                None => {
                                    let auth_task = get_authorization_task()?;
                                    let config =
                                        self.config.as_mut().ok_or("configuration not set")?;

                                    config.authorization_task = Some(auth_task);
                                    self.repo
                                        .send(repo::Request::GetFetcherConf(Some(config.clone())));
                                }
                            },
                            Some(auth) => {
                                if auth.expires_at < Utc::now() {
                                    let auth_task = get_authorization_task()?;
                                    let config =
                                        self.config.as_mut().ok_or("configuration not set")?;

                                    config.authorization = None;
                                    config.authorization_task = Some(auth_task);
                                    self.repo
                                        .send(repo::Request::GetFetcherConf(self.config.clone()));
                                } else {
                                    let mut url = get_url()?;

                                    match url.query_pairs().find(|(key, _)| key == "code") {
                                        Some(_) => {
                                            url.set_query(None);
                                            web_sys::window()
                                                .ok_or("could not get window")?
                                                .location()
                                                .set_href(url.as_str())?;
                                        }
                                        None => {}
                                    }
                                }
                            }
                        }
                    }
                    None => {
                        let url = get_base_url()?;

                        self.link.send_future(async move {
                            Message::GetConfig(
                                fetch_deserializable(
                                    &format!("{}/config/auth_config.json", url),
                                    HttpMethod::Get,
                                    None,
                                    None,
                                )
                                .await,
                            )
                        });
                    }
                },
                _ => {}
            },
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

        if let Some(config) = &self.config {
            if let Some(auth) = &config.authorization {
                let mut headers = HashMap::new();

                headers.insert(
                    "authorization".into(),
                    format!("Bearer {}", auth.access_token),
                );

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
                                fetch_deserializable(&url, HttpMethod::Get, Some(headers), None)
                                    .await,
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
                                fetch_deserializable(&url, HttpMethod::Get, Some(headers), None)
                                    .await,
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
                                fetch_deserializable(&url, HttpMethod::Get, Some(headers), None)
                                    .await,
                            )
                        });
                    }
                    Request::PullDownload(item_id) => {
                        let url = format!("/api/items/{}/stream", item_id);

                        self.link.send_future(async move {
                            Message::PullDownload(
                                id,
                                item_id,
                                fetch_binary(&url, Some(headers)).await,
                            )
                        });
                        self.link
                            .respond(id, Response::PullDownloadStarted(item_id));
                    }
                    Request::FetchBinary(uuid, url) => {
                        self.link.send_future(async move {
                            Message::ReceiveBinary(
                                id,
                                uuid,
                                fetch_binary(&url, Some(headers)).await,
                            )
                        });
                    }
                    Request::FetchText(uuid, url) => {
                        self.link.send_future(async move {
                            Message::ReceiveText(
                                id,
                                uuid,
                                fetch_text(&url, HttpMethod::Get, Some(headers), None).await,
                            )
                        });
                    }
                    Request::PostString(uuid, url, body) => {
                        self.link.send_future(async move {
                            Message::ReceiveText(
                                id,
                                uuid,
                                fetch_text(&url, HttpMethod::Post, Some(headers), Some(body)).await,
                            )
                        });
                    }
                }
            }
        }

        Ok(())
    }
}

impl Agent for Fetcher {
    type Reach = Context<Self>;
    type Message = Message;
    type Input = Request;
    type Output = Response;

    fn create(link: AgentLink<Self>) -> Self {
        let repo_callback = link.callback(Message::RepoMessage);
        let mut repo = repo::Repo::bridge(repo_callback);

        repo.send(repo::Request::GetFetcherConf(None));

        Self {
            link,
            subscribers: HashSet::<HandlerId>::new(),
            repo,
            config: None,
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

fn get_url() -> Result<Url, JsError> {
    Ok(Url::parse(
        &web_sys::window()
            .ok_or(JsError {
                description: String::from("error getting window"),
            })?
            .document()
            .ok_or(JsError {
                description: String::from("error getting document"),
            })?
            .url()?,
    )?)
}

fn get_base_url() -> Result<String, JsError> {
    let url = get_url()?;

    match (url.port(), url.host_str()) {
        (Some(port), Some(host)) => Ok(format!("{}://{}:{}", url.scheme(), host, port)),
        (None, Some(host)) => Ok(format!("{}://{}", url.scheme(), host)),
        (_, None) => Err(JsError {
            description: "no host string provided".into(),
        }),
    }
}

fn sha256_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();

    hasher.update(data);

    base64_url::encode(&hasher.finalize())
}

fn get_authorization_task() -> Result<AuthorizationTask, JsError> {
    let verifier = sha256_hash(Uuid::new_v4().as_bytes());
    let challenge = sha256_hash(verifier.as_bytes());

    Ok(AuthorizationTask {
        verifier,
        challenge,
        state: Uuid::new_v4(),
        redirect: get_base_url()?,
    })
}
