use crate::pages::{
    channels_page::ChannelsPage, feeds_page::FeedsPage, home_page::HomePage, items_page::ItemsPage,
};
use uuid::Uuid;
use yew::{prelude::*, virtual_dom::VNode};
use yew_router::Switch;

#[derive(Switch, Clone)]
pub enum AppRoute {
    #[to = "/channels/{channel_id}/items"]
    ItemsPage { channel_id: Uuid },
    #[to = "/channels"]
    ChannelsPage,
    #[to = "/feeds"]
    FeedsPage,
    #[to = "/"]
    Home,
}

pub struct Router {}
pub enum Message {}

impl Component for Router {
    type Message = Message;
    type Properties = ();

    fn view(&self) -> VNode {
        html! {
            <yew_router::router::Router<AppRoute>
                render = yew_router::router::Router::render(|switch: AppRoute| {
                    match switch {
                        AppRoute::Home => html!{<HomePage/>},
                        AppRoute::ChannelsPage => html!{<ChannelsPage/>},
                        AppRoute::FeedsPage => html!{<FeedsPage/>},
                        AppRoute::ItemsPage{channel_id} => html!{<ItemsPage channel_id={channel_id}/>},
                    }
                })
            />
        }
    }

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        todo!()
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        todo!()
    }
}
