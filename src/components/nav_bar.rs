use super::router::AppRoute;
use yew::{prelude::*, virtual_dom::VNode};
use yew_router::components::RouterAnchor;

pub struct NavBar {
    menu_expanded: bool,
    link: ComponentLink<Self>,
}
pub enum Message {
    ToggleMenu,
}

impl Component for NavBar {
    type Message = Message;
    type Properties = ();

    fn view(&self) -> VNode {
        let is_active_class = match self.menu_expanded {
            true => Some("is-active"),
            false => None,
        };

        html! {
            <nav class="navbar is-primary" role="navigation">
                <div class="navbar-brand">
                    <div class="navbar-item title">{"Podcast Player"}</div>
                    <a role="button" onclick={self.link.callback(|_| Message::ToggleMenu)} class={classes!("navbar-burger", is_active_class)} aria-label="menu" aria-expanded="false" data-target="navbarMenu">
                        <span aria-hidden="true"></span>
                        <span aria-hidden="true"></span>
                        <span aria-hidden="true"></span>
                    </a>
                </div>
                <div id="navbarMenu" class={classes!("navbar-menu", is_active_class)}>
                    <div class="navbar-start">
                        <RouterAnchor<AppRoute> classes={"navbar-item"} route={AppRoute::Home}>{"Home"}</RouterAnchor<AppRoute>>
                        <RouterAnchor<AppRoute> classes={"navbar-item"} route={AppRoute::ChannelsPage}>{"Channels"}</RouterAnchor<AppRoute>>
                        <RouterAnchor<AppRoute> classes={"navbar-item"} route={AppRoute::FeedsPage}>{"Feeds"}</RouterAnchor<AppRoute>>
                    </div>
                    <div class="navbar-end">
                    </div>
                </div>
            </nav>
        }
    }

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            menu_expanded: false,
            link,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::ToggleMenu => {
                self.menu_expanded = !self.menu_expanded;
                true
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }
}
