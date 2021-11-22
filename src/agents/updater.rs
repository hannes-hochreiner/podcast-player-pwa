// use anyhow::Result;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use yew::worker::*;

use crate::objects::{channel::Channel, item::Item};

use super::{
    fetcher::{self},
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
}

enum Task {
    GetChannels,
    GetItems(Uuid),
}

impl Agent for Updater {
    type Reach = Context<Self>;
    type Message = Message;
    type Input = Request;
    type Output = Response;

    fn create(link: AgentLink<Self>) -> Self {
        let window = web_sys::window().unwrap();
        let callback_repo = link.callback(Message::RepoMessage);
        let callback_fetcher = link.callback(Message::FetcherMessage);
        let callback_interval = link.callback(Message::Interval);
        let closure_interval =
            Closure::wrap(
                Box::new(move |event: web_sys::Event| callback_interval.emit(event))
                    as Box<dyn Fn(_)>,
            );
        window
            .set_interval_with_callback_and_timeout_and_arguments(
                closure_interval.as_ref().unchecked_ref(),
                10_000,
                &js_sys::Array::new(),
            )
            .unwrap();

        Self {
            _link: link,
            subscribers: HashSet::new(),
            _closure_interval: closure_interval,
            repo: repo::Repo::bridge(callback_repo),
            fetcher: fetcher::Fetcher::bridge(callback_fetcher),
            pending_tasks: HashMap::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) {
        match msg {
            Message::Interval(_ev) => {
                let task_id = Uuid::new_v4();

                self.pending_tasks.insert(task_id, Task::GetChannels);
                self.fetcher
                    .send(fetcher::Request::FetchText(task_id, "/api/channels".into()));
            }
            Message::RepoMessage(_) => {}
            Message::FetcherMessage(fm) => match fm {
                fetcher::Response::Text(task_id, res) => {
                    let task = self.pending_tasks.remove(&task_id).unwrap();

                    match res {
                        Ok(s) => match task {
                            Task::GetChannels => {
                                let channels: Vec<Channel> = serde_json::from_str(&s).unwrap();

                                for channel in &channels {
                                    let task_id = Uuid::new_v4();

                                    self.pending_tasks
                                        .insert(task_id, Task::GetItems(channel.id.clone()));
                                    self.fetcher.send(fetcher::Request::FetchText(
                                        task_id,
                                        format!("/api/channels/{}/items", channel.id).into(),
                                    ));
                                }
                                self.repo.send(repo::Request::AddChannels(channels));
                            }
                            Task::GetItems(_) => {
                                let items: Vec<Item> = serde_json::from_str(&s).unwrap();
                                self.repo.send(repo::Request::AddItems(items));
                            }
                        },
                        Err(_) => todo!("implement error handling"),
                    }
                }
                _ => {}
            },
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
