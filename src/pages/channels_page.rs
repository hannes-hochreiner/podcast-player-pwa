use crate::components::{channel_list::ChannelList, nav_bar::NavBar};
use yew::{prelude::*, virtual_dom::VNode};

pub struct ChannelsPage {}
pub enum Message {}

impl Component for ChannelsPage {
    type Message = Message;
    type Properties = ();

    fn view(&self) -> VNode {
        html! {
            <>
                <NavBar/>
                <ChannelList/>
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
