use super::router::AppRoute;
use yew::{prelude::*, virtual_dom::VNode};
use yew_router::components::RouterAnchor;

pub struct NavBar {}
pub enum Message {}

impl Component for NavBar {
    type Message = Message;
    type Properties = ();

    fn view(&self) -> VNode {
        html! {
            <nav class="navbar is-primary" role="navigation">
                <div class="navbar-brand">
                    <div class="navbar-item title">{"Podcast Player"}</div>
                    <a role="button" class="navbar-burger" aria-label="menu" aria-expanded="false">
                        <span aria-hidden="true"></span>
                        <span aria-hidden="true"></span>
                        <span aria-hidden="true"></span>
                    </a>
                </div>
                <div class="navbar-menu">
                    <div class="navbar-start">
                        <RouterAnchor<AppRoute> classes={"navbar-item"} route={AppRoute::Home}>{"Home"}</RouterAnchor<AppRoute>>
                        <RouterAnchor<AppRoute> classes={"navbar-item"} route={AppRoute::ChannelList}>{"Channels"}</RouterAnchor<AppRoute>>
                    </div>
                    <div class="navbar-end">
                    </div>
                </div>
            </nav>
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
