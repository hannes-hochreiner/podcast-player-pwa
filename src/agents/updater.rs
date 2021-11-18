// use anyhow::Result;
use std::collections::HashSet;
// use uuid::Uuid;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use yew::worker::*;

pub enum Request {}

pub enum Response {}

pub enum Message {
    Interval(web_sys::Event),
}

pub struct Updater {
    _link: AgentLink<Self>,
    subscribers: HashSet<HandlerId>,
    _closure_interval: Closure<dyn Fn(web_sys::Event)>,
}

impl Agent for Updater {
    type Reach = Context<Self>;
    type Message = Message;
    type Input = Request;
    type Output = Response;

    fn create(link: AgentLink<Self>) -> Self {
        let window = web_sys::window().unwrap();
        let callback_interval = link.callback(Message::Interval);
        let closure_interval =
            Closure::wrap(
                Box::new(move |event: web_sys::Event| callback_interval.emit(event))
                    as Box<dyn Fn(_)>,
            );
        window
            .set_interval_with_callback_and_timeout_and_arguments(
                closure_interval.as_ref().unchecked_ref(),
                5_000,
                &js_sys::Array::new(),
            )
            .unwrap();

        Self {
            _link: link,
            subscribers: HashSet::new(),
            _closure_interval: closure_interval,
        }
    }

    fn update(&mut self, msg: Self::Message) {
        match msg {
            Message::Interval(_ev) => {
                log::info!("timer elapsed")
            }
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
