use crate::components::{feed_list::FeedList, feed_new::FeedNew, nav_bar::NavBar};
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
