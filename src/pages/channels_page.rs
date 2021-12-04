use crate::components::{channel_list::ChannelList, router::AppRoute};
use yew::{prelude::*, virtual_dom::VNode};
use yew_router::components::RouterAnchor;

pub struct ChannelsPage {}
pub enum Message {}

impl Component for ChannelsPage {
    type Message = Message;
    type Properties = ();

    fn view(&self) -> VNode {
        html! {<ion-app>
            <ion-menu side="start" content-id="main-content">
                <ion-header>
                  <ion-toolbar color="primary">
                    <ion-title>{"Start Menu"}</ion-title>
                  </ion-toolbar>
                </ion-header>
                <ion-content>
                  <ion-list>
                    <ion-item><RouterAnchor<AppRoute> route={AppRoute::Home}>{"Home"}</RouterAnchor<AppRoute>></ion-item>
                    <ion-item>{"Channel"}</ion-item>
                  </ion-list>
                </ion-content>
            </ion-menu>
            <div class="ion-page" id="main-content">
            <ion-header>
                <ion-toolbar>
                    <ion-buttons slot="start">
                        <ion-menu-button></ion-menu-button>
                    </ion-buttons>
                    <ion-title>{"Podcast Player"}</ion-title>
                </ion-toolbar>
            </ion-header>
            <ion-content>
                <ChannelList/>
            </ion-content>
            </div>
        </ion-app>}
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
