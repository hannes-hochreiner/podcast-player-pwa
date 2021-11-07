use crate::pages::{channels_page::ChannelsPage, home_page::HomePage};
use yew::{prelude::*, virtual_dom::VNode};
use yew_router::Switch;

#[derive(Switch, Clone)]
pub enum AppRoute {
    #[to = "/channels"]
    ChannelList,
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
                        AppRoute::ChannelList => html!{<ChannelsPage/>}
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
