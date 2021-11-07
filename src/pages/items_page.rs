use crate::components::{item_list::ItemList, nav_bar::NavBar};
use uuid::Uuid;
use yew::{prelude::*, virtual_dom::VNode, Properties};

pub struct ItemsPage {
    channel_id: Uuid,
}
pub enum Message {}
#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub channel_id: Uuid,
}

impl Component for ItemsPage {
    type Message = Message;
    type Properties = Props;

    fn view(&self) -> VNode {
        html! {
            <>
                <NavBar/>
                <ItemList channel_id={self.channel_id}/>
            </>
        }
    }

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self {
            channel_id: props.channel_id,
        }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }
}
