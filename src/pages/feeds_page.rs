use crate::components::{feed_list::FeedList, feed_new::FeedNew, nav_bar::NavBar};
use yew::{prelude::*, virtual_dom::VNode};

pub struct FeedsPage {}
pub enum Message {}

impl Component for FeedsPage {
    type Message = Message;
    type Properties = ();

    fn view(&self) -> VNode {
        html! {
            <>
                <NavBar/>
                <FeedNew/>
                <FeedList/>
            </>
        }
    }

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }
}
