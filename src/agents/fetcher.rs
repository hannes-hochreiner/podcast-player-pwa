use crate::objects::{
    Auth0Token, Authorization, AuthorizationConfig, AuthorizationTask, FetcherConfig, JsError,
};

use super::repo;
use chrono::{Duration, Utc};
use js_sys::ArrayBuffer;
use serde::de::DeserializeOwned;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use url::Url;
use uuid::Uuid;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use yew_agent::{Agent, AgentLink, Bridge, Bridged, Context, HandlerId};

pub enum Request {
    FetchText(Uuid, String),
    FetchBinary(Uuid, String),
    PostString(Uuid, String, String),
}

pub enum Response {
    Binary(Uuid, Result<ArrayBuffer, JsError>),
    Text(Uuid, Result<String, JsError>),
}

pub enum Message {
    ReceiveText(HandlerId, Uuid, Result<String, JsError>),
    ReceiveBinary(HandlerId, Uuid, Result<ArrayBuffer, JsError>),
    GetToken(Result<Auth0Token, JsError>),
    RepoMessage(repo::Response),
    GetConfig(Result<AuthorizationConfig, JsError>),
}

pub struct Fetcher {
    link: AgentLink<Self>,
    subscribers: HashSet<HandlerId>,
    repo: Box<dyn Bridge<repo::Repo>>,
    config: Option<FetcherConfig>,
}

enum HttpMethod {
    Get,
    Post,
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
            Message::GetToken(res) => match (&mut self.config, res) {
                (Some(config), Ok(response)) => {
                    let mut config = config.clone();
                    config.authorization_task = None;
                    config.authorization = Some(Authorization {
                        access_token: response.access_token,
                        token_type: response.token_type,
                        expires_at: (Utc::now() + Duration::seconds(response.expires_in - 10))
                            .into(),
                    });
                    self.repo.send(repo::Request::GetFetcherConf(Some(config)))
                }
                (_, _) => {}
            },
            Message::GetConfig(res) => match res {
                Ok(conf) => {
                    self.repo.send(repo::Request::GetFetcherConf(Some(
                        FetcherConfig::new_with_config(conf),
                    )));
                }
                Err(e) => log::error!("{}", e),
            },
            Message::RepoMessage(repo_msg) => match repo_msg {
                repo::Response::FetcherConfig(fetcher_config) => match fetcher_config {
                    Some(fc) => {
                        self.config = Some(fc.clone());

                        match fc.authorization {
                            None => match &self.config.as_mut().unwrap().authorization_task {
                                Some(at) => {
                                    let url = get_url();

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
                                            ))
                                            .unwrap();
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
                                                .unwrap()
                                                .location()
                                                .set_href(url.as_str());
                                        }
                                    }
                                }
                                None => {
                                    self.config.as_mut().unwrap().authorization_task =
                                        Some(get_authorization_task());
                                    self.repo
                                        .send(repo::Request::GetFetcherConf(self.config.clone()));
                                }
                            },
                            Some(auth) => {
                                if auth.expires_at < Utc::now() {
                                    let config = self.config.as_mut().unwrap();

                                    config.authorization = None;
                                    config.authorization_task = Some(get_authorization_task());
                                    self.repo
                                        .send(repo::Request::GetFetcherConf(self.config.clone()));
                                }
                            }
                        }
                    }
                    None => {
                        self.link.send_future(async move {
                            Message::GetConfig(
                                fetch_deserializable(
                                    &format!("{}/config/auth_config.json", get_base_url()),
                                    HttpMethod::Get,
                                    None,
                                    None,
                                )
                                .await,
                            )
                        });
                    }
                },
                repo::Response::Error(err) => {
                    log::error!("{:?}", err);
                }
                _ => {}
            },
        }
    }

    fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
        if let Some(config) = &self.config {
            if let Some(auth) = &config.authorization {
                let mut headers = HashMap::new();

                headers.insert(
                    "authorization".into(),
                    format!("Bearer {}", auth.access_token),
                );

                match msg {
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
    }

    fn connected(&mut self, id: HandlerId) {
        self.subscribers.insert(id);
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
    }
}

async fn fetch(
    url: &str,
    method: HttpMethod,
    headers: Option<HashMap<String, String>>,
    body: Option<String>,
) -> Result<web_sys::Response, wasm_bindgen::JsValue> {
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
        opts.body(Some(&serde_wasm_bindgen::to_value(&val).unwrap()));
    }

    let request = web_sys::Request::new_with_str_and_init(url, &opts)?;
    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: web_sys::Response = resp_value.dyn_into().unwrap();

    Ok(resp)
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

fn get_url() -> Url {
    Url::parse(
        &web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .url()
            .unwrap(),
    )
    .unwrap()
}

fn get_base_url() -> String {
    let url = get_url();

    match url.port() {
        Some(port) => format!("{}://{}:{}", url.scheme(), url.host_str().unwrap(), port),
        None => format!("{}://{}", url.scheme(), url.host_str().unwrap()),
    }
}

fn sha256_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();

    hasher.update(data);

    base64_url::encode(&hasher.finalize())
}

fn get_authorization_task() -> AuthorizationTask {
    let verifier = sha256_hash(Uuid::new_v4().as_bytes());
    let challenge = sha256_hash(verifier.as_bytes());

    AuthorizationTask {
        verifier,
        challenge,
        state: Uuid::new_v4(),
        redirect: get_base_url(),
    }
}
