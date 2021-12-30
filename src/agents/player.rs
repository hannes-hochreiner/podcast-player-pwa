mod task;

use super::{notifier, repo};
use crate::objects::{Item, JsError};
use std::collections::HashSet;
use task::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::{closure::Closure, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{self, Event, MediaSource, Url};
use yew_agent::{Agent, AgentLink, Bridge, Bridged, Context, Dispatched, Dispatcher, HandlerId};

// TODO: check play events
// TODO: check whether any of the updated items need to be added to the item list

#[derive(Debug, Clone)]
pub enum Request {
    SetSource(Item),
    SetCurrentTime(f64),
    SetVolume(f64),
    SetPlaybackRate(f64),
    Play,
    Pause,
}

#[derive(Debug, Clone)]
pub enum Response {
    Playing,
    Paused,
    SourceSet(Item, f64),
    End,
}

#[derive(Debug)]
pub enum Message {
    RepoMessage(repo::Response),
    SourceOpened(Event),
    SourceBufferUpdate(Event),
    StartedPlaying(Result<JsValue, JsValue>),
    OnPlay(Event),
    OnPause(Event),
    OnTimeupdate(Event),
    OnEnd(Event),
}

pub struct Player {
    link: AgentLink<Self>,
    subscribers: HashSet<HandlerId>,
    repo: Box<dyn Bridge<repo::Repo>>,
    audio_element: web_sys::HtmlAudioElement,
    media_source: MediaSource,
    mediasource_opened_closure: Closure<dyn Fn(web_sys::Event)>,
    sourcebuffer_update_closure: Closure<dyn Fn(web_sys::Event)>,
    _on_play_closure: Closure<dyn Fn(web_sys::Event)>,
    _on_pause_closure: Closure<dyn Fn(web_sys::Event)>,
    _on_end_closure: Closure<dyn Fn(web_sys::Event)>,
    on_timeupdate_closure: Closure<dyn Fn(web_sys::Event)>,
    source: Option<Item>,
    notifier: Dispatcher<notifier::Notifier>,
    tasks: Vec<Task>,
}

impl Player {
    fn process_tasks(&mut self) {
        if let Some(mut task) = self.tasks.pop() {
            match self.process_task(&mut task) {
                Ok(response) => match response {
                    true => self.process_tasks(),
                    false => self.tasks.push(task),
                },
                Err(e) => {
                    self.notifier.send(notifier::Request::NotifyError(e));
                    self.process_tasks();
                }
            }
        }
    }

    fn process_task(&mut self, task: &mut Task) -> Result<bool, JsError> {
        match task {
            Task::End(task) => match task.get_stage() {
                EndStage::Finalize => {
                    log::info!(
                        "finalize: paused: {}, curr time: {}, duration: {}",
                        self.audio_element.paused(),
                        self.audio_element.current_time(),
                        self.audio_element.duration()
                    );
                    if let Some(item) = &mut self.source {
                        item.increment_play_count();
                        item.set_current_time(None);
                        self.repo.send(repo::Request::UpdateItem(item.clone()));
                        log::info!("send end");
                        self.send_response(Response::End);
                    }

                    Ok(true)
                }
            },
            Task::SetCurrentTime(task) => match task.get_stage() {
                SetCurrentTimeStage::Init => {
                    self.audio_element
                        .set_current_time(task.get_current_time().clone());
                    Ok(false)
                }
                SetCurrentTimeStage::Finalize => {
                    if let Some(item) = &mut self.source {
                        item.set_current_time(Some(self.audio_element.current_time()));
                        self.repo.send(repo::Request::UpdateItem(item.clone()));
                    }
                    Ok(true)
                }
            },
            Task::Pause(task) => match task.get_stage() {
                PauseStage::Init => {
                    if self.audio_element.paused() {
                        Ok(true)
                    } else {
                        self.audio_element.pause()?;
                        task.pause_triggered();
                        Ok(false)
                    }
                }
                PauseStage::WaitingForPause => Ok(false),
                PauseStage::Finalize => {
                    if let Some(curr_item) = &mut self.source {
                        self.audio_element.remove_event_listener_with_callback(
                            "timeupdate",
                            self.on_timeupdate_closure.as_ref().unchecked_ref(),
                        )?;
                        curr_item.set_current_time(Some(self.audio_element.current_time()));
                        self.repo.send(repo::Request::UpdateItem(curr_item.clone()));
                        self.send_response(Response::Paused);
                    }
                    Ok(true)
                }
            },
            Task::Play(task) => match task.get_stage() {
                PlayStage::Init => {
                    self.audio_element.add_event_listener_with_callback(
                        "timeupdate",
                        self.on_timeupdate_closure.as_ref().unchecked_ref(),
                    )?;

                    let prom = self.audio_element.play()?;

                    self.link.send_future(async move {
                        Message::StartedPlaying(JsFuture::from(prom).await)
                    });
                    task.play_triggered();
                    Ok(false)
                }
                PlayStage::WaitingForPlay => Ok(false),
                PlayStage::Finalize => {
                    self.send_response(Response::Playing);
                    Ok(true)
                }
            },
            Task::SetSource(task) => {
                match task.get_stage() {
                    SetSourceStage::Init => {
                        // remove source
                        self.source = None;
                        // set new media source
                        self.media_source = MediaSource::new()?;
                        self.audio_element
                            .set_src(&Url::create_object_url_with_source(&self.media_source)?);
                        self.media_source.set_onsourceopen(Some(
                            self.mediasource_opened_closure.as_ref().unchecked_ref(),
                        ));
                        // request data
                        self.repo
                            .send(repo::Request::GetEnclosure(task.get_item_ref().get_id()));
                        task.source_open_data_triggered();
                        Ok(false)
                    }
                    SetSourceStage::WaitingForSourceOpenData => Ok(false),
                    SetSourceStage::SourceOpenData => {
                        // clear existing source buffers
                        let sbl = self.media_source.source_buffers();

                        while let Some(sb) = sbl.get(0) {
                            self.media_source.remove_source_buffer(&sb)?
                        }
                        // create new source buffer
                        let sb = self.media_source.add_source_buffer("audio/mpeg")?;
                        self.audio_element.set_preload("metadata");
                        // load data
                        sb.append_buffer_with_array_buffer(
                            task.get_data_ref().ok_or("could not get data")?,
                        )?;
                        sb.set_onupdate(Some(
                            self.sourcebuffer_update_closure.as_ref().unchecked_ref(),
                        ));
                        task.data_buffer_update_triggered();
                        Ok(false)
                    }
                    SetSourceStage::WaitingForBufferUpdate => Ok(false),
                    SetSourceStage::Finalize => {
                        self.audio_element.set_playback_rate(1.5);
                        self.audio_element.set_volume(1.0);
                        self.audio_element.set_current_time(
                            match task.get_item_ref().get_current_time() {
                                Some(current_time) => current_time,
                                None => 0.0,
                            },
                        );
                        self.source = Some(task.get_item_ref().clone());
                        self.send_response(Response::SourceSet(
                            task.get_item_ref().clone(),
                            self.audio_element.duration(),
                        ));
                        Ok(true)
                    }
                }
            }
        }
    }

    fn send_response(&self, response: Response) {
        for handler_id in &self.subscribers {
            self.link.respond(*handler_id, response.clone())
        }
    }

    fn process_update(&mut self, msg: Message) -> Result<(), JsError> {
        match msg {
            Message::StartedPlaying(res) => {
                res?;
                if let Some(mut task) = self.tasks.last_mut() {
                    match &mut task {
                        Task::Play(task) => task.playing(),
                        _ => {}
                    }
                }
            }
            Message::RepoMessage(msg) => match msg {
                repo::Response::Enclosure(array_buffer) => {
                    if let Some(mut task) = self.tasks.last_mut() {
                        match &mut task {
                            Task::SetSource(task) => task.set_data(array_buffer),
                            _ => {}
                        }
                    }
                }
                repo::Response::ModifiedItems(items) => {
                    if let Some(source) = &self.source {
                        if let Some(item) = items.iter().find(|&i| i.get_id() == source.get_id()) {
                            self.source = Some((*item).clone())
                        }
                    }
                }
                _ => {}
            },
            Message::SourceOpened(_e) => {
                if let Some(mut task) = self.tasks.last_mut() {
                    match &mut task {
                        Task::SetSource(task) => task.source_opened(),
                        _ => {}
                    }
                }
            }
            Message::SourceBufferUpdate(_e) => {
                self.media_source.end_of_stream()?;
                if let Some(mut task) = self.tasks.last_mut() {
                    match &mut task {
                        Task::SetSource(task) => task.buffer_updated(),
                        _ => {}
                    }
                }
            }
            Message::OnEnd(_e) => {
                self.tasks.insert(0, Task::End(EndTask::new()));
            }
            Message::OnPause(_e) => {
                let mut task_required = false;

                if let Some(mut task) = self.tasks.last_mut() {
                    match &mut task {
                        Task::Pause(task) => {
                            task.paused();
                        }
                        _ => task_required = true,
                    }
                } else {
                    task_required = true;
                }

                if task_required {
                    let mut task = PauseTask::new();

                    task.paused();
                    self.tasks.insert(0, Task::Pause(task));
                }
            }
            Message::OnPlay(_e) => {
                let mut task_required = false;

                if let Some(mut task) = self.tasks.last_mut() {
                    match &mut task {
                        Task::Play(task) => {
                            task.playing();
                        }
                        _ => task_required = true,
                    }
                } else {
                    task_required = true;
                }

                if task_required {
                    let mut task = PlayTask::new();

                    task.playing();
                    self.tasks.insert(0, Task::Play(task));
                }
            }
            Message::OnTimeupdate(_e) => {
                let mut task_required = false;

                if let Some(mut task) = self.tasks.last_mut() {
                    match &mut task {
                        Task::SetCurrentTime(task) => {
                            task.time_set();
                        }
                        _ => task_required = true,
                    }
                } else {
                    task_required = true;
                }

                if task_required {
                    let mut task = SetCurrentTimeTask::new(self.audio_element.current_time());

                    task.time_set();
                    self.tasks.insert(0, Task::SetCurrentTime(task));
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

        // set audio element callbacks
        let on_play_callback = link.callback(move |e| Message::OnPlay(e));
        let on_play_closure =
            Closure::wrap(
                Box::new(move |event: web_sys::Event| on_play_callback.emit(event))
                    as Box<dyn Fn(_)>,
            );
        let on_pause_callback = link.callback(move |e| Message::OnPause(e));
        let on_pause_closure =
            Closure::wrap(
                Box::new(move |event: web_sys::Event| on_pause_callback.emit(event))
                    as Box<dyn Fn(_)>,
            );
        let on_timeupdate_callback = link.callback(move |e| Message::OnTimeupdate(e));
        let on_timeupdate_closure =
            Closure::wrap(
                Box::new(move |event: web_sys::Event| on_timeupdate_callback.emit(event))
                    as Box<dyn Fn(_)>,
            );
        let on_end_callback = link.callback(move |e| Message::OnEnd(e));
        let on_end_closure =
            Closure::wrap(
                Box::new(move |event: web_sys::Event| on_end_callback.emit(event))
                    as Box<dyn Fn(_)>,
            );
        let audio_element = web_sys::HtmlAudioElement::new().unwrap();

        audio_element
            .add_event_listener_with_callback("play", on_play_closure.as_ref().unchecked_ref())
            .unwrap();
        audio_element
            .add_event_listener_with_callback("pause", on_pause_closure.as_ref().unchecked_ref())
            .unwrap();
        audio_element
            .add_event_listener_with_callback("ended", on_end_closure.as_ref().unchecked_ref())
            .unwrap();
        // not sure how one could avoid the unwraps in the object creation; on option might be
        // to wrap the media source and audio element in options; however, this increases the
        // complexity and still does not provide the required functionality
        Self {
            link,
            audio_element,
            subscribers: HashSet::new(),
            repo: repo::Repo::bridge(callback_repo),
            media_source: MediaSource::new().unwrap(),
            mediasource_opened_closure,
            sourcebuffer_update_closure,
            notifier: notifier::Notifier::dispatcher(),
            source: None,
            tasks: Vec::new(),
            _on_pause_closure: on_pause_closure,
            _on_play_closure: on_play_closure,
            _on_end_closure: on_end_closure,
            on_timeupdate_closure: on_timeupdate_closure,
        }
    }

    fn update(&mut self, msg: Self::Message) {
        match self.process_update(msg) {
            Ok(()) => {}
            Err(e) => self.notifier.send(notifier::Request::NotifyError(e)),
        }

        self.process_tasks();
    }

    fn handle_input(&mut self, msg: Self::Input, _handler_id: HandlerId) {
        match msg {
            Request::SetSource(item) => {
                self.tasks.insert(0, Task::Pause(PauseTask::new()));
                self.tasks
                    .insert(0, Task::SetSource(SetSourceTask::new(item)));
            }
            Request::Play => self.tasks.insert(0, Task::Play(PlayTask::new())),
            Request::Pause => self.tasks.insert(0, Task::Pause(PauseTask::new())),
            Request::SetCurrentTime(time) => self
                .tasks
                .insert(0, Task::SetCurrentTime(SetCurrentTimeTask::new(time))),
            _ => {}
        }
        self.process_tasks();
    }

    fn connected(&mut self, id: HandlerId) {
        self.subscribers.insert(id);
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
    }
}
