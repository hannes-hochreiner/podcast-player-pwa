use crate::agents::repo::{Repo, Request, Response};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{MediaSource, Url};
use yew::{prelude::*, virtual_dom::VNode};

pub struct Player {
    _repo: Box<dyn Bridge<Repo>>,
    link: ComponentLink<Self>,
    source: String,
    update_closure: Closure<dyn Fn(web_sys::Event)>,
    media_source: MediaSource,
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
        let callback_update = link.callback(move |e| Message::Update(e));
        let closure_update =
            Closure::wrap(
                Box::new(move |event: web_sys::Event| callback_update.emit(event))
                    as Box<dyn Fn(_)>,
            );

        Self {
            link,
            _repo: Repo::bridge(cb),
            source: Url::create_object_url_with_source(&ms).unwrap(),
            media_source: ms,
            audio_ref: NodeRef::default(),
            update_closure: closure_update,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::NewMessage(response) => match response {
                Response::Enclosure(res) => {
                    if let Ok(data) = res {
                        let sb = self.media_source.add_source_buffer("audio/mpeg").unwrap();
                        let ae = self.audio_ref.cast::<web_sys::HtmlAudioElement>().unwrap();
                        ae.set_playback_rate(2.0);
                        ae.set_preload("metadata");
                        sb.append_buffer_with_array_buffer(&data).unwrap();
                        sb.set_onupdate(Some(self.update_closure.as_ref().unchecked_ref()));
                    }
                    true
                }
                _ => false,
            },
            Message::Play => {
                self._repo.send(Request::GetEnclosure(
                    uuid::Uuid::parse_str("200541bb-662b-40b9-a2d8-7d38444216f6").unwrap(),
                ));
                false
            }
            Message::Info => {
                let ae = self.audio_ref.cast::<web_sys::HtmlAudioElement>().unwrap();

                log::info!(
                    "preload: {} ms: {}, time: {}, ae state: {}, ms state: {:?}",
                    ae.preload(),
                    self.media_source.duration(),
                    ae.current_time(),
                    ae.ready_state(),
                    self.media_source.ready_state()
                );
                false
            }
            Message::Update(e) => {
                log::info!("update: {:?}", e);

                self.media_source.end_of_stream().unwrap();
                false
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }
}
