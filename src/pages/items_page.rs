use crate::components::{ItemList, NavBar};
use uuid::Uuid;
use yew::{prelude::*, Html, Properties};

pub struct ItemsPage {}
pub enum Message {}
#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub channel_id: Uuid,
}

impl Component for ItemsPage {
    type Message = Message;
    type Properties = Props;

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
                <NavBar/>
                <ItemList channel_id={ctx.props().channel_id}/>
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
