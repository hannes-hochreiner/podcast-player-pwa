use wasm_bindgen::JsCast;
use web_sys::{Event, FocusEvent, HtmlInputElement};
use yew::{html, Callback, Component, Properties};
use yew_agent::{Dispatched, Dispatcher};

use crate::{agents::notifier, objects::JsError};

pub struct Range {
    update_allowed: bool,
    notifier: Dispatcher<notifier::Notifier>,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub onchange: Callback<String>,
    pub value: String,
    pub min: String,
    pub max: String,
    pub step: String,
}

pub enum Message {
    OnFocus(FocusEvent),
    OnChange(Event),
}

impl Range {
    fn process_update(&mut self, ctx: &yew::Context<Self>, msg: Message) -> Result<bool, JsError> {
        match msg {
            Message::OnChange(event) => {
                let ie = get_input_element_from_event(event)?;
                ie.blur()?;
                ctx.props().onchange.emit(ie.value());
                self.update_allowed = true;
                Ok(false)
            }
            Message::OnFocus(_focus_event) => {
                self.update_allowed = false;
                Ok(false)
            }
        }
    }
}

impl Component for Range {
    type Message = Message;
    type Properties = Props;

    fn create(_ctx: &yew::Context<Self>) -> Self {
        Self {
            update_allowed: true,
            notifier: notifier::Notifier::dispatcher(),
        }
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        html! {
            <input type="range" min={ctx.props().min.clone()} step={ctx.props().step.clone()} value={ctx.props().value.clone()} max={ctx.props().max.clone()} style="width: 100%" onfocus={ctx.link().callback(|e| Message::OnFocus(e))} onchange={ctx.link().callback(|e| Message::OnChange(e))}/>
        }
    }

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match self.process_update(ctx, msg) {
            Ok(should_render) => should_render,
            Err(e) => {
                self.notifier.send(notifier::Request::NotifyError(e));
                false
            }
        }
    }

    fn changed(&mut self, _ctx: &yew::Context<Self>) -> bool {
        self.update_allowed
    }
}

fn get_input_element_from_event(ev: Event) -> Result<HtmlInputElement, JsError> {
    let target = ev.target().ok_or("could not get target object")?;

    target
        .dyn_into::<HtmlInputElement>()
        .map_err(|_| JsError::from("error casting target to input element"))
}
