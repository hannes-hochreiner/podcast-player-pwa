use super::router::AppRoute;
use yew::{prelude::*, virtual_dom::VNode};
use yew_router::prelude::*;

pub struct NavBar {
    menu_expanded: bool,
}
pub enum Message {
    ToggleMenu,
}

impl Component for NavBar {
    type Message = Message;
    type Properties = ();

    fn view(&self, ctx: &Context<Self>) -> VNode {
        let is_active_class = match self.menu_expanded {
            true => Some("is-active"),
            false => None,
        };

        html! {
            <nav class="navbar is-primary" role="navigation">
                <div class="navbar-brand">
                    <div class="navbar-item title">{"Podcast Player"}</div>
                    <a role="button" onclick={ctx.link().callback(|_| Message::ToggleMenu)} class={classes!("navbar-burger", is_active_class)} aria-label="menu" aria-expanded="false" data-target="navbarMenu">
                        <span aria-hidden="true"></span>
                        <span aria-hidden="true"></span>
                        <span aria-hidden="true"></span>
                    </a>
                </div>
                <div id="navbarMenu" class={classes!("navbar-menu", is_active_class)}>
                    <div class="navbar-start">
                        <Link<AppRoute> classes={"navbar-item"} to={AppRoute::Home}>{"Home"}</Link<AppRoute>>
                        <Link<AppRoute> classes={"navbar-item"} to={AppRoute::ChannelsPage}>{"Channels"}</Link<AppRoute>>
                        <Link<AppRoute> classes={"navbar-item"} to={AppRoute::FeedsPage}>{"Feeds"}</Link<AppRoute>>
                    </div>
                    <div class="navbar-end">
                    </div>
                </div>
            </nav>
        }
    }

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            menu_expanded: false,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::ToggleMenu => {
                self.menu_expanded = !self.menu_expanded;
                true
            }
        }
    }
}
