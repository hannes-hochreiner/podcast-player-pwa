// use anyhow::Result;
use crate::objects::{ChannelVal, DownloadStatus, FeedVal, ItemVal, JsError};
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
}

enum Task {
    GetFeeds,
    GetChannels,
    GetItems(Uuid),
}

impl Updater {
    fn process_update(&mut self, msg: Message) -> Result<(), JsError> {
        match msg {
            Message::Interval(_ev) => {
                let conn_type = web_sys::window()
                    .ok_or("could not obtain window")?
                    .navigator()
                    .connection()?
                    .type_();
                match conn_type {
                    ConnectionType::Ethernet | ConnectionType::Wifi | ConnectionType::Unknown => {
                        let task_id_feeds = Uuid::new_v4();

                        self.pending_tasks.insert(task_id_feeds, Task::GetFeeds);
                        self.fetcher.send(fetcher::Request::FetchText(
                            task_id_feeds,
                            "/api/feeds".into(),
                        ));

                        let task_id_channels = Uuid::new_v4();

                        self.pending_tasks
                            .insert(task_id_channels, Task::GetChannels);
                        self.fetcher.send(fetcher::Request::FetchText(
                            task_id_channels,
                            "/api/channels".into(),
                        ));
                    }
                    _ => {}
                }
            }
            Message::RepoMessage(msg) => match msg {
                repo::Response::AddChannelVals(_) => {
                    self.repo.send(repo::Request::GetItemsByDownloadRequired);
                }
                repo::Response::Items(items) => {
                    if items.len() > 0 {
                        let mut item = items[0].clone();

                        item.set_download_status(DownloadStatus::InProgress);

                        self.repo
                            .send(repo::Request::DownloadEnclosure(item.get_id()));
                        self.repo.send(repo::Request::UpdateItem(item));
                    }
                }
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

                                self.repo.send(repo::Request::AddFeedVals(feeds));
                            }
                            Task::GetChannels => {
                                let channels: Vec<ChannelVal> = serde_json::from_str(&s)?;

                                for channel in &channels {
                                    let task_id = Uuid::new_v4();

                                    self.pending_tasks
                                        .insert(task_id, Task::GetItems(channel.id.clone()));
                                    self.fetcher.send(fetcher::Request::FetchText(
                                        task_id,
                                        format!("/api/channels/{}/items", channel.id).into(),
                                    ));
                                }
                                self.repo.send(repo::Request::AddChannelVals(channels));
                            }
                            Task::GetItems(_) => {
                                let items: Vec<ItemVal> = serde_json::from_str(&s)?;
                                self.repo.send(repo::Request::AddItemVals(items));
                            }
                        },
                        Err(_) => todo!("implement error handling"),
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
                        Err(_) => todo!("implement error handling"),
                    }
                }
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

        Self {
            _link: link,
            subscribers: HashSet::new(),
            _closure_interval: closure_interval,
            repo: repo::Repo::bridge(callback_repo),
            fetcher: fetcher::Fetcher::bridge(callback_fetcher),
            pending_tasks: HashMap::new(),
            notifier: notifier,
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
