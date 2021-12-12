use crate::objects::{
    auth0_token::Auth0Token,
    fetcher_config::{Authorization, AuthorizationTask, FetcherConfig},
};

use super::repo;
use anyhow::Result;
use chrono::{Duration, Utc};
use js_sys::ArrayBuffer;
use serde::de::DeserializeOwned;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use url::Url;
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
    Binary(Uuid, anyhow::Result<ArrayBuffer>),
    Text(Uuid, anyhow::Result<String>),
}

pub enum Message {
    ReceiveText(HandlerId, Uuid, Result<String>),
    ReceiveBinary(HandlerId, Uuid, Result<ArrayBuffer>),
    GetToken(Result<Auth0Token>),
    RepoMessage(repo::Response),
}

pub struct Fetcher {
    link: AgentLink<Self>,
    subscribers: HashSet<HandlerId>,
    client_id: String,
    audience: String,
    domain: String,
    repo: Box<dyn Bridge<repo::Repo>>,
    config: Option<FetcherConfig>,
}

enum HttpMethod {
    Get,
    Post,
}

impl Fetcher {
    fn sha256_hash(&self, data: &[u8]) -> String {
        let mut hasher = Sha256::new();

        hasher.update(data);

        base64_url::encode(&hasher.finalize())
    }

    fn get_authorization_task(&self) -> AuthorizationTask {
        let verifier = self.sha256_hash(Uuid::new_v4().as_bytes());
        let challenge = self.sha256_hash(verifier.as_bytes());

        AuthorizationTask {
            verifier,
            challenge,
            state: Uuid::new_v4(),
            redirect: "http://127.0.0.1:8080".into(),
        }
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
            audience: env!("AUDIENCE").into(),
            client_id: env!("CLIENT_ID").into(),
            domain: env!("DOMAIN").into(),
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
            Message::GetToken(res) => {
                log::info!("{:?}", res);

                match (&mut self.config, res) {
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
                }
            }
            Message::RepoMessage(repo_msg) => match repo_msg {
                repo::Response::FetcherConfig(fetcher_config) => match fetcher_config {
                    Some(fc) => {
                        self.config = Some(fc);

                        log::info!("obtained fetcher config");

                        if self.config.as_ref().unwrap().authorization.is_none() {
                            match &self.config.as_mut().unwrap().authorization_task {
                                Some(at) => {
                                    let url = Url::parse(
                                        &web_sys::window()
                                            .unwrap()
                                            .document()
                                            .unwrap()
                                            .url()
                                            .unwrap(),
                                    )
                                    .unwrap();

                                    match url.query_pairs().find(|(key, val)| key == "code") {
                                        Some((key, val)) => {
                                            log::info!("found code: {}", val);
                                            let token_url = format!("{}/oauth/token", self.domain);
                                            let body = format!("grant_type=authorization_code&client_id={}&code_verifier={}&code={}&redirect_uri={}", self.client_id, at.verifier, val, at.redirect);
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
                                            let mut url =
                                                Url::parse(&format!("{}/authorize", self.domain))
                                                    .unwrap();
                                            let verifier = url
                                                .query_pairs_mut()
                                                .append_pair("audience", &self.audience)
                                                .append_pair("scope", "profile")
                                                .append_pair("response_type", "code")
                                                .append_pair("client_id", &self.client_id)
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
                                        Some(self.get_authorization_task());
                                    self.repo
                                        .send(repo::Request::GetFetcherConf(self.config.clone()));
                                }
                            }
                        }
                    }
                    None => {
                        self.repo.send(repo::Request::GetFetcherConf(Some(
                            FetcherConfig::default(),
                        )));
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
        match msg {
            Request::FetchBinary(uuid, url) => {
                self.link.send_future(async move {
                    Message::ReceiveBinary(id, uuid, fetch_binary(&url).await)
                });
            }
            Request::FetchText(uuid, url) => {
                self.link.send_future(async move {
                    Message::ReceiveText(
                        id,
                        uuid,
                        fetch_text(&url, HttpMethod::Get, None, None).await,
                    )
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
        let opt_headers = web_sys::Headers::new().unwrap();

        for (key, val) in headers {
            opt_headers.append(&key, &val);
        }

        opts.headers(&opt_headers);
    }

    if let Some(val) = body {
        opts.body(Some(&serde_wasm_bindgen::to_value(&val).unwrap()));
    }

    let request = web_sys::Request::new_with_str_and_init(url, &opts)?;
    let window = yew::utils::window();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: web_sys::Response = resp_value.dyn_into().unwrap();

    Ok(resp)
}

// https://github.com/yewstack/yew/blob/v0.18/examples/futures/src/main.rs
async fn fetch_binary(url: &str) -> Result<ArrayBuffer> {
    Ok(ArrayBuffer::from(
        JsFuture::from(
            fetch(url, HttpMethod::Get, None, None)
                .await
                .map_err(|_| anyhow::anyhow!("fetch failed"))?
                .array_buffer()
                .map_err(|_| anyhow::anyhow!("retrieving arraybuffer failed"))?,
        )
        .await
        .map_err(|_| anyhow::anyhow!("creating future failed"))?,
    ))
}

async fn fetch_text(
    url: &str,
    method: HttpMethod,
    headers: Option<HashMap<String, String>>,
    body: Option<String>,
) -> anyhow::Result<String> {
    JsFuture::from(
        fetch(url, method, headers, body)
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

async fn fetch_deserializable<T: DeserializeOwned>(
    url: &str,
    method: HttpMethod,
    headers: Option<HashMap<String, String>>,
    body: Option<String>,
) -> anyhow::Result<T> {
    JsFuture::from(
        fetch(url, method, headers, body)
            .await
            .map_err(|_| anyhow::anyhow!("fetch failed"))?
            .json()
            .map_err(|_| anyhow::anyhow!("retrieving json failed"))?,
    )
    .await
    .map_err(|_| anyhow::anyhow!("creating future failed"))
    .map(|val| {
        serde_wasm_bindgen::from_value(val)
            .map_err(|e| anyhow::anyhow!("parsing failed: {}", e.to_string()))
    })?
}
