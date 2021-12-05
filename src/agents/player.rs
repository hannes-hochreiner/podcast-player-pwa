use std::collections::HashSet;
use uuid::Uuid;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{self, Event, MediaSource, Url};
use yew::worker::*;

use super::repo;

pub enum Request {
    Play {
        id: Uuid,
        current_time: f64,
        volume: f64,
        playback_rate: f64,
    },
    Pause,
}

pub enum Response {
    Playing {
        id: Uuid,
        duration: f64,
        current_time: f64,
    },
}

pub enum Message {
    RepoMessage(repo::Response),
    SourceOpened(Event),
    SourceBufferUpdate(Event),
    Interval(web_sys::Event),
}

pub struct Player {
    link: AgentLink<Self>,
    subscribers: HashSet<HandlerId>,
    repo: Box<dyn Bridge<repo::Repo>>,
    active_task: Option<Task>,
    audio_element: web_sys::HtmlAudioElement,
    media_source: MediaSource,
    mediasource_opened_closure: Closure<dyn Fn(web_sys::Event)>,
    sourcebuffer_update_closure: Closure<dyn Fn(web_sys::Event)>,
    interval_closure: Closure<dyn Fn(web_sys::Event)>,
    active_id: Option<Uuid>,
    interval_handle: Option<i32>,
}

enum Task {
    Play {
        handler_id: HandlerId,
        request: Request,
    },
}

impl Player {
    fn start_setting_source(&mut self, id: Uuid, handler_id: HandlerId, msg: Request) {
        self.active_id = Some(id);
        self.active_task = Some(Task::Play {
            handler_id,
            request: msg,
        });
        self.media_source = MediaSource::new().unwrap();
        self.audio_element
            .set_src(&Url::create_object_url_with_source(&self.media_source).unwrap());
        self.media_source.set_onsourceopen(Some(
            self.mediasource_opened_closure.as_ref().unchecked_ref(),
        ));
    }
}

impl Agent for Player {
    type Reach = Context<Self>;
    type Message = Message;
    type Input = Request;
    type Output = Response;

    fn create(link: AgentLink<Self>) -> Self {
        let callback_repo = link.callback(Message::RepoMessage);
        let callback_mediasource_opened = link.callback(move |e| Message::SourceOpened(e));
        let mediasource_opened_closure = Closure::wrap(Box::new(move |event: web_sys::Event| {
            callback_mediasource_opened.emit(event)
        }) as Box<dyn Fn(_)>);
        let callback_sourcebuffer_update = link.callback(move |e| Message::SourceBufferUpdate(e));
        let sourcebuffer_update_closure = Closure::wrap(Box::new(move |event: web_sys::Event| {
            callback_sourcebuffer_update.emit(event)
        }) as Box<dyn Fn(_)>);
        let callback_interval = link.callback(Message::Interval);
        let interval_closure =
            Closure::wrap(
                Box::new(move |event: web_sys::Event| callback_interval.emit(event))
                    as Box<dyn Fn(_)>,
            );

        Self {
            link,
            audio_element: web_sys::HtmlAudioElement::new().unwrap(),
            subscribers: HashSet::new(),
            repo: repo::Repo::bridge(callback_repo),
            active_task: None,
            media_source: MediaSource::new().unwrap(),
            mediasource_opened_closure,
            active_id: None,
            sourcebuffer_update_closure,
            interval_closure,
            interval_handle: None,
        }
    }

    fn update(&mut self, msg: Self::Message) {
        match msg {
            Message::Interval(_e) => {
                if let Some(id) = self.active_id {
                    for handler_id in &self.subscribers {
                        self.link.respond(
                            *handler_id,
                            Response::Playing {
                                id: id.clone(),
                                duration: self.audio_element.duration(),
                                current_time: self.audio_element.current_time(),
                            },
                        );
                    }
                }
            }
            Message::RepoMessage(msg) => match msg {
                repo::Response::Enclosure(data) => {
                    let sb = self.media_source.add_source_buffer("audio/mpeg").unwrap();
                    self.audio_element.set_playback_rate(1.5);
                    self.audio_element.set_preload("metadata");
                    sb.append_buffer_with_array_buffer(&data).unwrap();
                    sb.set_onupdate(Some(
                        self.sourcebuffer_update_closure.as_ref().unchecked_ref(),
                    ));
                }
                _ => {}
            },
            Message::SourceOpened(_e) => match self.active_id {
                Some(id) => self.repo.send(repo::Request::GetEnclosure(id)),
                None => {}
            },
            Message::SourceBufferUpdate(_e) => {
                self.media_source.end_of_stream().unwrap();
                self.audio_element.play();

                match (&self.active_task, &self.active_id) {
                    (Some(task), Some(id)) => match task {
                        Task::Play {
                            handler_id,
                            request,
                        } => {
                            match request {
                                &Request::Play {
                                    current_time,
                                    playback_rate: speed,
                                    volume,
                                    id: _,
                                } => {
                                    self.audio_element.set_playback_rate(speed);
                                    self.audio_element.set_volume(volume);
                                    self.audio_element.set_current_time(current_time);
                                }
                                _ => {}
                            }

                            self.link.respond(
                                handler_id.clone(),
                                Response::Playing {
                                    id: id.clone(),
                                    duration: self.audio_element.duration(),
                                    current_time: self.audio_element.current_time(),
                                },
                            );
                            let window = web_sys::window().unwrap();

                            self.interval_handle = Some(
                                window
                                    .set_interval_with_callback_and_timeout_and_arguments(
                                        self.interval_closure.as_ref().unchecked_ref(),
                                        1_000,
                                        &js_sys::Array::new(),
                                    )
                                    .unwrap(),
                            );
                        }
                    },
                    (_, _) => {}
                }
            }
        }
    }

    fn handle_input(&mut self, msg: Self::Input, handler_id: HandlerId) {
        match msg {
            Request::Play {
                id,
                current_time,
                volume,
                playback_rate,
            } => match self.active_id {
                Some(active_id) => match active_id == id {
                    true => {
                        self.audio_element.set_playback_rate(playback_rate);
                        self.audio_element.set_volume(volume);
                        self.audio_element.set_current_time(current_time);
                    }
                    false => self.start_setting_source(id, handler_id, msg),
                },
                None => {
                    self.start_setting_source(id, handler_id, msg);
                }
            },
            Request::Pause => {
                self.audio_element.pause().unwrap();

                if let Some(interval_handle) = self.interval_handle {
                    let window = web_sys::window().unwrap();

                    window.clear_interval_with_handle(interval_handle);
                    self.interval_handle = None;
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
