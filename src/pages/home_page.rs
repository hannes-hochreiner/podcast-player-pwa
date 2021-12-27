use crate::components::{NavBar, Notification, Player};
use yew::{prelude::*, Html};

pub struct HomePage {}
pub enum Message {}

impl Component for HomePage {
    type Message = Message;
    type Properties = ();

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <>
                <NavBar/>
                <Notification/>
                <Player/>
            </>
        }
    }

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }
}
