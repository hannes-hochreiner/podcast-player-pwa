use super::router::Router;
use crate::agents::{player, updater};
use yew::{prelude::*, Component};

pub struct Top {
    _updater: Box<dyn Bridge<updater::Updater>>,
    _player: Box<dyn Bridge<player::Player>>,
}
pub enum Message {
    UpdaterMessage(updater::Response),
    PlayerMessage(player::Response),
}

impl Component for Top {
    type Message = Message;
    type Properties = ();

    fn create(_props: Self::Properties, link: yew::ComponentLink<Self>) -> Self {
        let updater_cb = link.callback(Message::UpdaterMessage);
        let player_cb = link.callback(Message::PlayerMessage);

        Self {
            _updater: updater::Updater::bridge(updater_cb),
            _player: player::Player::bridge(player_cb),
        }
    }

    fn update(&mut self, msg: Self::Message) -> yew::ShouldRender {
        match msg {
            Message::UpdaterMessage(_resp) => false,
            Message::PlayerMessage(_resp) => false,
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
