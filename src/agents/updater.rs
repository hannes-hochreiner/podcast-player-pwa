use crate::objects::{ChannelVal, DownloadStatus, FeedVal, ItemVal, JsError, UpdaterConfig};
use crate::utils;
use chrono::Utc;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::ConnectionType;
use yew_agent::{Agent, AgentLink, Bridge, Bridged, Context, Dispatched, Dispatcher, HandlerId};

use super::{
    fetcher::{self},
    notifier::{self},
    repo::{self},
};

pub enum Request {}

pub enum Response {}

pub enum Message {
    Interval(web_sys::Event),
    FetcherMessage(fetcher::Response),
    RepoMessage(repo::Response),
}

pub struct Updater {
    _link: AgentLink<Self>,
    subscribers: HashSet<HandlerId>,
    _closure_interval: Closure<dyn Fn(web_sys::Event)>,
    repo: Box<dyn Bridge<repo::Repo>>,
    fetcher: Box<dyn Bridge<fetcher::Fetcher>>,
    pending_tasks: HashMap<Uuid, Task>,
    notifier: Dispatcher<notifier::Notifier>,
    config: Option<UpdaterConfig>,
}

enum Task {
    GetFeeds,
    GetChannels,
    GetItems,
}

impl Updater {
    fn process_update(&mut self, msg: Message) -> Result<(), JsError> {
        match msg {
            Message::Interval(_ev) => {
                let conn_type = utils::get_connection_type()?;

                match (&self.config, conn_type) {
                    (
                        Some(config),
                        ConnectionType::Ethernet | ConnectionType::Wifi | ConnectionType::Unknown,
                    ) => {
                        let task_id_feeds = Uuid::new_v4();
                        let url_feeds = String::from("/api/feeds");

                        self.pending_tasks.insert(task_id_feeds, Task::GetFeeds);
                        self.fetcher.send(fetcher::Request::FetchText(
                            task_id_feeds,
                            match config.last_fetch_feeds {
                                Some(date) => {
                                    let encoded: String =
                                        url::form_urlencoded::Serializer::new(String::new())
                                            .append_pair("since", &date.to_rfc3339())
                                            .finish();
                                    format!("{}?{}", url_feeds, encoded)
                                }
                                None => url_feeds,
                            },
                        ));

                        let task_id_channels = Uuid::new_v4();
                        let url_channels = String::from("/api/channels");

                        self.pending_tasks
                            .insert(task_id_channels, Task::GetChannels);
                        self.fetcher.send(fetcher::Request::FetchText(
                            task_id_channels,
                            match config.last_fetch_channels {
                                Some(date) => {
                                    let encoded: String =
                                        url::form_urlencoded::Serializer::new(String::new())
                                            .append_pair("since", &date.to_rfc3339())
                                            .finish();
                                    format!("{}?{}", url_channels, encoded)
                                }
                                None => url_channels,
                            },
                        ));

                        let task_id_items = Uuid::new_v4();
                        let url_items = String::from("/api/items");

                        self.pending_tasks.insert(task_id_items, Task::GetItems);
                        self.fetcher.send(fetcher::Request::FetchText(
                            task_id_items,
                            match config.last_fetch_items {
                                Some(date) => {
                                    let encoded: String =
                                        url::form_urlencoded::Serializer::new(String::new())
                                            .append_pair("since", &date.to_rfc3339())
                                            .finish();
                                    format!("{}?{}", url_items, encoded)
                                }
                                None => url_items,
                            },
                        ));
                    }
                    _ => {}
                }
            }
            Message::RepoMessage(msg) => match msg {
                // repo::Response::AddChannelVals(_) => {
                //     self.repo.send(repo::Request::GetItemsByDownloadRequired);
                // }
                repo::Response::Items(items) => {
                    if items.len() > 0 {
                        let mut item = items[0].clone();

                        item.set_download_status(DownloadStatus::InProgress);

                        // self.repo
                        //     .send(repo::Request::DownloadEnclosure(item.get_id()));
                        self.repo.send(repo::Request::UpdateItem(item));
                    }
                }
                repo::Response::UpdaterConfig(config) => match config {
                    Some(config) => {
                        if self.config.is_none() {
                            self.config = Some(config);
                        }
                    }
                    None => self
                        .repo
                        .send(repo::Request::GetUpdaterConf(Some(UpdaterConfig {
                            last_fetch_feeds: None,
                            last_fetch_channels: None,
                            last_fetch_items: None,
                        }))),
                },
                _ => {}
            },
            Message::FetcherMessage(fm) => match fm {
                fetcher::Response::Text(task_id, res) => {
                    let task = self
                        .pending_tasks
                        .remove(&task_id)
                        .ok_or("task not found")?;

                    match res {
                        Ok(s) => match task {
                            Task::GetFeeds => {
                                let feeds: Vec<FeedVal> = serde_json::from_str(&s)?;

                                if let Some(config) = &mut self.config {
                                    config.last_fetch_feeds = Some(Utc::now().into());
                                    self.repo
                                        .send(repo::Request::GetUpdaterConf(Some(config.clone())))
                                }
                                // self.repo.send(repo::Request::AddFeedVals(feeds));
                            }
                            Task::GetChannels => {
                                let channels: Vec<ChannelVal> = serde_json::from_str(&s)?;

                                if let Some(config) = &mut self.config {
                                    config.last_fetch_channels = Some(Utc::now().into());
                                    self.repo
                                        .send(repo::Request::GetUpdaterConf(Some(config.clone())))
                                }
                                // self.repo.send(repo::Request::AddChannelVals(channels));
                            }
                            Task::GetItems => {
                                let items: Vec<ItemVal> = serde_json::from_str(&s)?;

                                if let Some(config) = &mut self.config {
                                    config.last_fetch_items = Some(Utc::now().into());
                                    self.repo
                                        .send(repo::Request::GetUpdaterConf(Some(config.clone())))
                                }
                                // self.repo.send(repo::Request::AddItemVals(items));
                            }
                        },
                        Err(e) => self.notifier.send(notifier::Request::NotifyError(e)),
                    }
                }
                fetcher::Response::Binary(task_id, res) => {
                    let task = self
                        .pending_tasks
                        .remove(&task_id)
                        .ok_or("could not find task")?;

                    match res {
                        Ok(_ab) => match task {
                            _ => {}
                        },
                        Err(e) => self.notifier.send(notifier::Request::NotifyError(e)),
                    }
                }
                _ => {}
            },
        };
        Ok(())
    }
}

impl Agent for Updater {
    type Reach = Context<Self>;
    type Message = Message;
    type Input = Request;
    type Output = Response;

    fn create(link: AgentLink<Self>) -> Self {
        let mut notifier = notifier::Notifier::dispatcher();
        let callback_repo = link.callback(Message::RepoMessage);
        let callback_fetcher = link.callback(Message::FetcherMessage);
        let callback_interval = link.callback(Message::Interval);
        let closure_interval =
            Closure::wrap(
                Box::new(move |event: web_sys::Event| callback_interval.emit(event))
                    as Box<dyn Fn(_)>,
            );
        match web_sys::window()
            .ok_or(JsError::from_str("could not obtain window"))
            .and_then(|w| {
                w.set_interval_with_callback_and_timeout_and_arguments(
                    closure_interval.as_ref().unchecked_ref(),
                    10_000,
                    &js_sys::Array::new(),
                )
                .map_err(Into::into)
            }) {
            Ok(_handle) => {}
            Err(e) => notifier.send(notifier::Request::NotifyError(e)),
        }

        let mut repo = repo::Repo::bridge(callback_repo);

        repo.send(repo::Request::GetUpdaterConf(None));

        Self {
            _link: link,
            subscribers: HashSet::new(),
            _closure_interval: closure_interval,
            repo,
            fetcher: fetcher::Fetcher::bridge(callback_fetcher),
            pending_tasks: HashMap::new(),
            notifier: notifier,
            config: None,
        }
    }

    fn update(&mut self, msg: Self::Message) {
        match self.process_update(msg) {
            Ok(()) => {}
            Err(e) => self.notifier.send(notifier::Request::NotifyError(e)),
        }
    }

    fn handle_input(&mut self, _msg: Self::Input, _id: HandlerId) {
        todo!()
    }

    fn connected(&mut self, id: HandlerId) {
        self.subscribers.insert(id);
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
    }
}
