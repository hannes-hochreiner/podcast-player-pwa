use crate::components::{ChannelList, NavBar, Notification};
use yew::{prelude::*, Html};

pub struct ChannelsPage {}
pub enum Message {}

impl Component for ChannelsPage {
    type Message = Message;
    type Properties = ();

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <>
                <NavBar/>
                // <Notification/>
                <ChannelList/>
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
