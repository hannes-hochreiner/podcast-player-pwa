use super::router::Router;
use crate::agents::updater;
use yew::{prelude::*, Component};

pub struct Top {
    _updater: Box<dyn Bridge<updater::Updater>>,
}
pub enum Message {
    UpdaterMessage(updater::Response),
}

impl Component for Top {
    type Message = Message;
    type Properties = ();

    fn create(_props: Self::Properties, link: yew::ComponentLink<Self>) -> Self {
        let cb = link.callback(Message::UpdaterMessage);

        Self {
            _updater: updater::Updater::bridge(cb),
        }
    }

    fn update(&mut self, msg: Self::Message) -> yew::ShouldRender {
        match msg {
            Message::UpdaterMessage(_resp) => false,
        }
    }

    fn change(&mut self, _props: Self::Properties) -> yew::ShouldRender {
        todo!()
    }

    fn view(&self) -> yew::Html {
        html! {
            <Router/>
        }
    }
}
