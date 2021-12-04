use super::router::AppRoute;
use crate::agents::repo::{Repo, Request as RepoRequest, Response as RepoResponse};
use crate::objects::channel::Channel;
use anyhow::Error;
use uuid::Uuid;
use yew::prelude::*;
use yew_router::prelude::RouterAnchor;

pub struct ChannelList {
    link: ComponentLink<Self>,
    channels: Option<Vec<Channel>>,
    error: Option<Error>,
    repo: Box<dyn Bridge<Repo>>,
    filter: Filter,
}

pub enum Message {
    RepoMessage(RepoResponse),
    SetActive(Uuid, bool),
    FilterChange(Filter),
}

pub enum Filter {
    Selected,
    All,
}

impl ChannelList {
    fn view_channel_list(&self) -> Html {
        match &self.channels {
            Some(channels) => {
                html! {<>
                    <ion-segment value="selected">
                        <ion-segment-button value="selected" onclick={self.link.callback(move |_| Message::FilterChange(Filter::Selected))}><ion-label>{"Selected"}</ion-label></ion-segment-button>
                        <ion-segment-button value="all" onclick={self.link.callback(move |_| Message::FilterChange(Filter::All))}><ion-label>{"All"}</ion-label></ion-segment-button>
                    </ion-segment>
                    <ion-list>
                        {match self.filter {
                            Filter::Selected => {
                                channels.iter().filter(|&e| e.meta.active).map(|channel|
                                    self.view_show_selected_channel(channel)
                                ).collect::<Html>()
                            },
                            Filter::All => {
                                channels.iter().map(|channel|
                                    self.view_show_all_channel(channel)
                                ).collect::<Html>()
                            }
                        }}
                    </ion-list>
                </>}
            }
            _ => html! { <p> {"no channels available"} </p> },
        }
    }

    fn view_show_selected_channel(&self, channel: &Channel) -> Html {
        html! { <RouterAnchor<AppRoute> route={AppRoute::ItemsPage{channel_id: channel.val.id}}>
            <ion-item>
                <ion-thumbnail slot="start">
                    <ion-img src={channel.val.image.clone()}></ion-img>
                </ion-thumbnail>
                <ion-label>{&channel.val.title}</ion-label>
            </ion-item>
        </RouterAnchor<AppRoute>> }
    }

    fn view_show_all_channel(&self, channel: &Channel) -> Html {
        let state = channel.meta.active;
        let channel_id = channel.val.id.clone();

        html! {
            <ion-item button="true" color={match state {
                true => "secondary",
                false => ""
            }} onclick={self.link.callback(move |_| Message::SetActive(channel_id, !state))}>
                <ion-thumbnail slot="start">
                    <ion-img src={channel.val.image.clone()}></ion-img>
                </ion-thumbnail>
                <ion-label>
                    <ion-title>{&channel.val.title}</ion-title>
                    <ion-sub-title>{&channel.val.description}</ion-sub-title>
                </ion-label>
            </ion-item>
        }
    }

    fn view_fetching(&self) -> Html {
        // if self.fetch_task.is_some() {
        //     html! { <p>{ "Fetching data..." }</p> }
        // } else {
        html! {}
        // }
    }

    fn view_error(&self) -> Html {
        match &self.error {
            Some(e) => html! { <p>{e}</p> },
            None => html! {},
        }
    }
}

impl Component for ChannelList {
    type Message = Message;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let cb = link.callback(Message::RepoMessage);
        let mut repo = Repo::bridge(cb);

        repo.send(RepoRequest::GetChannels);

        Self {
            link: link,
            channels: None,
            error: None,
            repo,
            filter: Filter::Selected,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::RepoMessage(response) => match response {
                RepoResponse::Channels(res) => {
                    self.channels = Some(res);
                    true
                }
                RepoResponse::Error(e) => {
                    log::info!("channel list error: {}", e);
                    false
                }
                _ => false,
            },
            Message::SetActive(id, state) => {
                log::info!("set active called: id: {}, state: {}", id, state);
                let mut channel = self
                    .channels
                    .as_ref()
                    .unwrap()
                    .iter()
                    .find(|e| e.val.id == id)
                    .unwrap()
                    .clone();
                channel.meta.active = state;
                self.repo.send(RepoRequest::UpdateChannel(channel));
                false
            }
            Message::FilterChange(filter) => {
                self.filter = filter;
                true
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <>
                { self.view_fetching() }
                { self.view_channel_list() }
                { self.view_error() }
            </>
        }
    }
}
