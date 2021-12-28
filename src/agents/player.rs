use super::{notifier, repo};
use crate::objects::JsError;
use std::collections::HashSet;
use uuid::Uuid;
use wasm_bindgen::JsCast;
use wasm_bindgen::{closure::Closure, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{self, Event, MediaSource, Url};
use yew_agent::{Agent, AgentLink, Bridge, Bridged, Context, Dispatched, Dispatcher, HandlerId};

// TODO: implement task list to ensure serial execution of tasks (especially to avoid interrupting the set source task)

pub enum Request {
    SetSource {
        id: Uuid,
        current_time: f64,
        volume: f64,
        playback_rate: f64,
    },
    SetCurrentTime(f64),
    SetVolume(f64),
    SetPlaybackRate(f64),
    Play,
    Pause,
}

pub enum Response {
    Update {
        id: Uuid,
        duration: f64,
        current_time: f64,
        volume: f64,
        playback_rate: f64,
        is_playing: bool,
    },
}

pub enum Message {
    RepoMessage(repo::Response),
    SourceOpened(Event),
    SourceBufferUpdate(Event),
    Interval(web_sys::Event),
    StartedPlaying(Result<JsValue, JsValue>),
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
    notifier: Dispatcher<notifier::Notifier>,
}

enum Task {
    SetSource { request: Request },
}

impl Player {
    fn process_handle_input(
        &mut self,
        msg: Request,
        _handler_id: HandlerId,
    ) -> Result<(), JsError> {
        match msg {
            Request::SetSource {
                id,
                current_time: _,
                volume: _,
                playback_rate: _,
            } => {
                if self.active_id.is_some() {
                    self.audio_element.pause()?;
                    self.remove_interval()?;
                    self.send_update();
                    self.active_id = None;
                }
                self.start_setting_source(id, msg)?;
            }
            Request::Play => {
                if self.active_id.is_some() {
                    let prom = self.audio_element.play()?;

                    self.link.send_future(async move {
                        Message::StartedPlaying(JsFuture::from(prom).await)
                    });
                }
            }
            Request::SetCurrentTime(current_time) => {
                if self.active_id.is_some() {
                    self.audio_element.set_current_time(current_time);
                }
            }
            Request::SetPlaybackRate(playback_rate) => {
                if self.active_id.is_some() {
                    self.audio_element.set_playback_rate(playback_rate);
                }
            }
            Request::SetVolume(volume) => {
                if self.active_id.is_some() {
                    self.audio_element.set_volume(volume);
                }
            }
            Request::Pause => {
                self.audio_element.pause()?;
                self.remove_interval()?;
                self.send_update();
            }
        }

        Ok(())
    }

    fn start_setting_source(&mut self, id: Uuid, msg: Request) -> Result<(), JsError> {
        self.active_id = Some(id);
        self.active_task = Some(Task::SetSource { request: msg });
        self.media_source = MediaSource::new()?;
        self.audio_element
            .set_src(&Url::create_object_url_with_source(&self.media_source)?);
        self.media_source.set_onsourceopen(Some(
            self.mediasource_opened_closure.as_ref().unchecked_ref(),
        ));
        Ok(())
    }

    fn set_interval(&mut self) -> Result<(), JsError> {
        match self.interval_handle {
            Some(_) => {}
            None => {
                let window = web_sys::window().ok_or("could not obtain window")?;

                self.interval_handle =
                    Some(window.set_interval_with_callback_and_timeout_and_arguments(
                        self.interval_closure.as_ref().unchecked_ref(),
                        1_000,
                        &js_sys::Array::new(),
                    )?);
            }
        }
        Ok(())
    }

    fn remove_interval(&mut self) -> Result<(), JsError> {
        if let Some(interval_handle) = self.interval_handle {
            let window = web_sys::window().ok_or("could not obtain window")?;

            window.clear_interval_with_handle(interval_handle);
            self.interval_handle = None;
        }
        Ok(())
    }

    fn send_update(&self) {
        if let Some(id) = self.active_id {
            for handler_id in &self.subscribers {
                self.link.respond(
                    *handler_id,
                    Response::Update {
                        id: id.clone(),
                        duration: self.audio_element.duration(),
                        current_time: self.audio_element.current_time(),
                        playback_rate: self.audio_element.playback_rate(),
                        volume: self.audio_element.volume(),
                        is_playing: !self.audio_element.paused(),
                    },
                );
            }
        }
    }

    fn process_update(&mut self, msg: Message) -> Result<(), JsError> {
        match msg {
            Message::StartedPlaying(res) => match res {
                Ok(_) => self.set_interval()?,
                Err(e) => self.notifier.send(notifier::Request::NotifyError(e.into())),
            },
            Message::Interval(_e) => self.send_update(),
            Message::RepoMessage(msg) => match msg {
                repo::Response::Enclosure(data) => {
                    let sb = self.media_source.add_source_buffer("audio/mpeg")?;
                    self.audio_element.set_preload("metadata");
                    sb.append_buffer_with_array_buffer(&data)?;
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
                self.media_source.end_of_stream()?;

                match &self.active_task {
                    Some(task) => match task {
                        Task::SetSource { request } => match request {
                            &Request::SetSource {
                                id: _,
                                volume,
                                playback_rate,
                                current_time,
                            } => {
                                self.audio_element.set_playback_rate(playback_rate);
                                self.audio_element.set_volume(volume);
                                self.audio_element.set_current_time(current_time);
                                self.send_update();
                            }
                            _ => {}
                        },
                    },
                    _ => {}
                }
            }
        }

        Ok(())
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

        // not sure how one could avoid the unwraps in the object creation; on option might be
        // to wrap the media source and audio element in options; however, this increases the
        // complexity and still does not provide the required functionality
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
            notifier: notifier::Notifier::dispatcher(),
        }
    }

    fn update(&mut self, msg: Self::Message) {
        match self.process_update(msg) {
            Ok(()) => {}
            Err(e) => self.notifier.send(notifier::Request::NotifyError(e)),
        }
    }

    fn handle_input(&mut self, msg: Self::Input, handler_id: HandlerId) {
        match self.process_handle_input(msg, handler_id) {
            Ok(()) => {}
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
