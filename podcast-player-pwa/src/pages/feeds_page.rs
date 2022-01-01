use crate::components::{FeedList, FeedNew, NavBar, Notification};
use yew::{prelude::*, Html};

pub struct FeedsPage {}
pub enum Message {}

impl Component for FeedsPage {
    type Message = Message;
    type Properties = ();

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <>
                <NavBar/>
                <Notification/>
                <FeedNew/>
                <FeedList/>
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
