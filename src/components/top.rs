use super::router::Router;
use crate::agents::player;
use yew::{prelude::*, Component};
use yew_agent::{Bridge, Bridged};
use yew_router::BrowserRouter;

pub struct Top {
    _player: Box<dyn Bridge<player::Player>>,
}
pub enum Message {
    PlayerMessage(player::Response),
}

impl Component for Top {
    type Message = Message;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let player_cb = ctx.link().callback(Message::PlayerMessage);

        Self {
            _player: player::Player::bridge(player_cb),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn view(&self, _ctx: &Context<Self>) -> yew::Html {
        html! {
            <BrowserRouter>
                <Router/>
            </BrowserRouter>
        }
    }
}
