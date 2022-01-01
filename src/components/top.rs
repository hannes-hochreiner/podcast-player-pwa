use super::router::Router;
use crate::agents::{player, updater};
use yew::{prelude::*, Component};
use yew_agent::{Bridge, Bridged};
use yew_router::BrowserRouter;

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

    fn create(ctx: &Context<Self>) -> Self {
        let updater_cb = ctx.link().callback(Message::UpdaterMessage);
        let player_cb = ctx.link().callback(Message::PlayerMessage);

        Self {
            _updater: updater::Updater::bridge(updater_cb),
            _player: player::Player::bridge(player_cb),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::UpdaterMessage(_resp) => false,
            Message::PlayerMessage(_resp) => false,
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> yew::Html {
        html! {
            <BrowserRouter>
                <Router/>
            </BrowserRouter>
        }
    }
}
