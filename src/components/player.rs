use crate::agents::repo::{Repo, Request, Response};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{MediaSource, Url};
use yew::{agent::Dispatcher, prelude::*, virtual_dom::VNode};

pub struct Player {
    _repo: Dispatcher<Repo>,
    _producer: Box<dyn Bridge<Repo>>,
    link: ComponentLink<Self>,
    source: String,
    update_closure: Option<Closure<dyn Fn(web_sys::Event)>>,
    media_source: Option<MediaSource>,
    audio_ref: NodeRef,
}
pub enum Message {
    NewMessage(Response),
    Play,
    Info,
    Update(Event),
}

impl Component for Player {
    type Message = Message;
    type Properties = ();

    fn view(&self) -> VNode {
        html! {
            <>
                <audio controls={true} src={self.source.clone()} ref=self.audio_ref.clone()/>
                <button class="button" onclick={self.link.callback(move |_| Message::Play)}>{"set source"}</button>
                <button class="button" onclick={self.link.callback(move |_| Message::Info)}>{"info"}</button>
            </>
        }
    }

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let cb = link.callback(Message::NewMessage);
        let ms = MediaSource::new().unwrap();

        Self {
            link,
            _repo: Repo::dispatcher(),
            _producer: Repo::bridge(cb),
            source: Url::create_object_url_with_source(&ms).unwrap(),
            media_source: Some(ms),
            audio_ref: NodeRef::default(),
            update_closure: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::NewMessage(response) => match response {
                Response::Channels(_) => true,
                Response::Enclosure(data) => {
                    match &self.media_source {
                        Some(ms) => {
                            let sb = ms.add_source_buffer("audio/mpeg").unwrap();
                            let ae = self.audio_ref.cast::<web_sys::HtmlAudioElement>().unwrap();
                            ae.set_playback_rate(2.0);
                            ae.set_preload("metadata");
                            sb.append_buffer_with_array_buffer(&data).unwrap();

                            let callback_update = self.link.callback(move |e| Message::Update(e));
                            let closure_update =
                                Closure::wrap(Box::new(move |event: web_sys::Event| {
                                    callback_update.emit(event)
                                }) as Box<dyn Fn(_)>);

                            sb.set_onupdate(Some(closure_update.as_ref().unchecked_ref()));
                            self.update_closure = Some(closure_update);

                            log::info!(
                                "duration: {} ms: {}, time: {}",
                                ae.preload(),
                                ms.duration(),
                                ae.current_time()
                            );
                        }
                        None => log::error!("no media source set"),
                    }
                    true
                }
            },
            Message::Play => {
                self._repo.send(Request::GetEnclosure(
                    uuid::Uuid::parse_str("200541bb-662b-40b9-a2d8-7d38444216f6").unwrap(),
                ));
                false
            }
            Message::Info => {
                match &self.media_source {
                    Some(ms) => {
                        let ae = self.audio_ref.cast::<web_sys::HtmlAudioElement>().unwrap();

                        log::info!(
                            "preload: {} ms: {}, time: {}, ae state: {}, ms state: {:?}",
                            ae.preload(),
                            ms.duration(),
                            ae.current_time(),
                            ae.ready_state(),
                            ms.ready_state()
                        );
                    }
                    None => log::error!("no media source set"),
                }
                false
            }
            Message::Update(e) => {
                log::info!("update: {:?}", e);

                if let Some(ms) = &self.media_source {
                    ms.end_of_stream().unwrap();
                }
                false
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }
}
