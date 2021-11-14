use crate::agents::repo::{Repo, Request, Response};
use js_sys::Uint8Array;
use wasm_bindgen::JsCast;
use wasm_bindgen::{closure::Closure, JsValue};
use web_sys::{AudioBuffer, AudioBufferSourceNode, AudioContext};
use yew::{agent::Dispatcher, prelude::*, virtual_dom::VNode};

pub struct Player {
    _repo: Dispatcher<Repo>,
    _producer: Box<dyn Bridge<Repo>>,
    link: ComponentLink<Self>,
    source: String,
    decode_task: Option<DecodeTask>,
    pipeline: Option<Pipeline>,
}
pub enum Message {
    NewMessage(Response),
    Play,
    DecodeError(Event),
    DecodeSuccess(Event),
}

struct Pipeline {
    buffer: AudioBuffer,
    source_node: AudioBufferSourceNode,
}

struct DecodeTask {
    _closure_error: Closure<dyn Fn(web_sys::Event)>,
    _closure_success: Closure<dyn Fn(web_sys::Event)>,
}

impl Component for Player {
    type Message = Message;
    type Properties = ();

    fn view(&self) -> VNode {
        html! {
            <>
                <audio controls={true} src={self.source.clone()}/>
                <button class="button" onclick={self.link.callback(move |_| Message::Play)}>{"set source"}</button>
            </>
        }
    }

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let cb = link.callback(Message::NewMessage);

        Self {
            link,
            _repo: Repo::dispatcher(),
            _producer: Repo::bridge(cb),
            source: String::new(),
            decode_task: None,
            pipeline: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::NewMessage(response) => match response {
                Response::Channels(_) => true,
                Response::Enclosure(data) => {
                    let ac = AudioContext::new().unwrap();
                    let callback_error = self.link.callback(move |e| Message::DecodeError(e));
                    let callback_success = self.link.callback(move |e| Message::DecodeSuccess(e));
                    let closure_success = Closure::wrap(Box::new(move |event: web_sys::Event| {
                        callback_success.emit(event)
                    }) as Box<dyn Fn(_)>);
                    let closure_error = Closure::wrap(Box::new(move |event: web_sys::Event| {
                        callback_error.emit(event)
                    }) as Box<dyn Fn(_)>);
                    let _ = ac
                        .decode_audio_data_with_success_callback_and_error_callback(
                            &data,
                            closure_success.as_ref().unchecked_ref(),
                            closure_error.as_ref().unchecked_ref(),
                        )
                        .unwrap();
                    self.decode_task = Some(DecodeTask {
                        _closure_error: closure_error,
                        _closure_success: closure_success,
                    });

                    true
                }
            },
            Message::Play => {
                self._repo.send(Request::GetEnclosure(
                    uuid::Uuid::parse_str("200541bb-662b-40b9-a2d8-7d38444216f6").unwrap(),
                ));
                false
            }
            Message::DecodeSuccess(e) => {
                log::info!("decode success: {:?}", e);
                let ab = AudioBuffer::from(JsValue::from(e));
                let ac = AudioContext::new().unwrap();
                let absn = AudioBufferSourceNode::new(&ac).unwrap();
                absn.set_buffer(Some(&ab));
                absn.connect_with_audio_node(&ac.destination()).unwrap();
                absn.start().unwrap();
                self.pipeline = Some(Pipeline {
                    buffer: ab,
                    source_node: absn,
                });
                true
            }
            Message::DecodeError(e) => {
                log::error!("decode error: {:?}", e);
                false
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }
}
