use crate::components::{item_list::ItemList, router::AppRoute};
use uuid::Uuid;
use yew::{prelude::*, virtual_dom::VNode, Properties};
use yew_router::components::RouterAnchor;

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
                <ItemList channel_id={self.channel_id}/>
            </ion-content>
            </div>
        </ion-app>}
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
