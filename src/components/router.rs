use crate::pages::{
    channels_page::ChannelsPage, feeds_page::FeedsPage, home_page::HomePage, info_page::InfoPage,
    items_page::ItemsPage,
};
use uuid::Uuid;
use yew::{prelude::*, Html};
use yew_router::prelude::*;

#[derive(Clone, Routable, PartialEq)]
pub enum AppRoute {
    #[at("/channels/{channel_id}/items")]
    ItemsPage { channel_id: Uuid },
    #[at("/channels")]
    ChannelsPage,
    #[at("/feeds")]
    FeedsPage,
    #[at("/info")]
    InfoPage,
    #[at("/")]
    Home,
}

pub struct Router {}
pub enum Message {}

impl Component for Router {
    type Message = Message;
    type Properties = ();

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <Switch<AppRoute> render={Switch::render(switch)} />
        }
    }

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        todo!()
    }
}

fn switch(routes: &AppRoute) -> Html {
    match routes.clone() {
        AppRoute::Home => html! {<HomePage/>},
        AppRoute::ChannelsPage => html! {<ChannelsPage/>},
        AppRoute::FeedsPage => html! {<FeedsPage/>},
        AppRoute::InfoPage => html! {<InfoPage/>},
        AppRoute::ItemsPage { channel_id } => html! {<ItemsPage channel_id={channel_id}/>},
    }
}
