use std::rc::Rc;

use crate::agents::updater::{self, Updater};

use super::router::Router;
// use crate::agents::player;
use yew::{prelude::*, Component};
use yew_agent::{Bridge, Bridged};
use yew_router::BrowserRouter;

pub struct Top {
    // _player: Box<dyn Bridge<player::Player>>,
    _updater: Box<dyn Bridge<Updater>>,
}
pub enum Message {
    // PlayerMessage(player::Response),
    UpdaterMessage(updater::Response),
}

impl Component for Top {
    type Message = Message;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        // let player_cb = ctx.link().callback(Message::PlayerMessage);
        let cb = {
            let link = ctx.link().clone();
            move |e| link.send_message(Self::Message::UpdaterMessage(e))
        };
        let updater = Updater::bridge(Rc::new(cb));

        Self {
            // _player: player::Player::bridge(player_cb),
            _updater: updater,
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
