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
    show_all: bool,
}

pub enum Message {
    RepoMessage(RepoResponse),
    SetShowAll(bool),
    SetActive(Uuid, bool),
}

impl ChannelList {
    fn view_channel_list(&self) -> Html {
        match &self.channels {
            Some(channels) => {
                let channels: Vec<&Channel> = match self.show_all {
                    true => channels.iter().collect(),
                    false => channels.iter().filter(|&e| e.meta.active).collect(),
                };

                html! {
                    <section class="section">
                        <div class="tabs is-toggle is-fullwidth">
                            <ul>
                                {match self.show_all {
                                    false => {
                                        html! {
                                            <>
                                                <li class="is-active"><a><span>{"Active"}</span></a></li>
                                                <li><a onclick={self.link.callback(move |_| Message::SetShowAll(true))}><span>{"All"}</span></a></li>
                                            </>
                                        }
                                    },
                                    true => {
                                        html! {
                                            <>
                                                <li><a onclick={self.link.callback(move |_| Message::SetShowAll(false))}><span>{"Active"}</span></a></li>
                                                <li class="is-active"><a><span>{"All"}</span></a></li>
                                            </>
                                        }
                                    },
                                }}
                            </ul>
                        </div>
                        <div class="columns"><div class="column">
                            { channels.iter().map(|channel| match self.show_all {
                                true => self.view_show_all_channel(channel),
                                false => self.view_show_selected_channel(channel)
                            }).collect::<Html>() }
                        </div></div>
                    </section>
                }
            }
            _ => html! { <p> {"no channels available"} </p> },
        }
    }

    fn view_show_selected_channel(&self, channel: &Channel) -> Html {
        html! { <RouterAnchor<AppRoute> classes={"navbar-item, card"} route={AppRoute::ItemsPage{channel_id: channel.val.id}}>
            <div class="card-content">
                <div class="media">
                    <div class="media-left"><figure class="image is-64x64"><img src={channel.val.image.clone()}/></figure></div>
                    <div class="media-content"><p class="title">{&channel.val.title}</p><p class="subtitle">{&channel.val.description}</p></div>
                </div>
            </div>
        </RouterAnchor<AppRoute>> }
    }

    fn view_show_all_channel(&self, channel: &Channel) -> Html {
        let state = channel.meta.active;
        let channel_id = channel.val.id.clone();

        html! {
            <div class="card-content">
                <div class="media">
                    <div class="media-left"><figure class="image is-64x64"><img src={channel.val.image.clone()}/></figure></div>
                    <div class="media-content">
                        <p class="title">{&channel.val.title}</p><p class="subtitle">{&channel.val.description}</p>
                        {match state {
                            true => html!(<button class="button is-primary" onclick={self.link.callback(move |_| Message::SetActive(channel_id, false))}><ion-icon size="large" name="checkmark" /></button>),
                            false => html!(<button class="button" onclick={self.link.callback(move |_| Message::SetActive(channel_id, true))}><ion-icon size="large" name="checkmark" /></button>)
                        }}
                    </div>
                </div>
            </div>
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
            show_all: false,
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
            Message::SetShowAll(show_all) => {
                self.show_all = show_all;
                true
            }
            Message::SetActive(id, state) => {
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
