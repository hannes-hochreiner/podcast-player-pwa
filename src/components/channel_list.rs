use std::collections::HashMap;

use super::router::AppRoute;
use crate::agents::repo::{Repo, Request as RepoRequest, Response as RepoResponse};
use crate::objects::channel::Channel;
use crate::objects::channel_meta::ChannelMeta;
use anyhow::Error;
use uuid::Uuid;
use yew::prelude::*;
use yew_router::prelude::RouterAnchor;

pub struct ChannelList {
    link: ComponentLink<Self>,
    channels: Option<Vec<Channel>>,
    channels_meta: Option<Vec<ChannelMeta>>,
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
        match (&self.channels, &self.channels_meta) {
            (Some(channels), Some(channels_meta)) => {
                let meta_map: HashMap<Uuid, &ChannelMeta> =
                    channels_meta.iter().map(|e| (e.id, e)).collect();
                let channels: Vec<(&Channel, &ChannelMeta)> = match self.show_all {
                    true => channels.iter().map(|e| (e, meta_map[&e.id])).collect(),
                    false => {
                        let selected: Vec<Uuid> = channels_meta
                            .iter()
                            .filter(|e| e.active)
                            .map(|e| e.id)
                            .collect();
                        channels
                            .iter()
                            .filter(|&e| selected.contains(&e.id))
                            .map(|e| (e, meta_map[&e.id]))
                            .collect()
                    }
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
                                true => self.view_show_all_channel(channel.0, channel.1),
                                false => self.view_show_selected_channel(channel.0, channel.1)
                            }).collect::<Html>() }
                        </div></div>
                    </section>
                }
            }
            _ => html! { <p> {"no channels available"} </p> },
        }
    }

    fn view_show_selected_channel(&self, channel: &Channel, _channel_meta: &ChannelMeta) -> Html {
        html! { <RouterAnchor<AppRoute> classes={"navbar-item, card"} route={AppRoute::ItemsPage{channel_id: channel.id}}>
            <div class="card-content">
                <div class="media">
                    <div class="media-left"><figure class="image is-64x64"><img src={channel.image.clone()}/></figure></div>
                    <div class="media-content"><p class="title">{&channel.title}</p><p class="subtitle">{&channel.description}</p></div>
                </div>
            </div>
        </RouterAnchor<AppRoute>> }
    }

    fn view_show_all_channel(&self, channel: &Channel, channel_meta: &ChannelMeta) -> Html {
        let state = channel_meta.active;
        let channel_id = channel.id.clone();

        html! {
            <div class="card-content">
                <div class="media">
                    <div class="media-left"><figure class="image is-64x64"><img src={channel.image.clone()}/></figure></div>
                    <div class="media-content"><p class="title">{&channel.title}</p><p class="subtitle">{&channel.description}</p></div>
                    <label class="checkbox">
                        <input type="checkbox" checked={state} oninput={self.link.callback(move |_| Message::SetActive(channel_id, !state))}/>
                    </label>
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
            channels_meta: None,
            error: None,
            repo,
            show_all: false,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::RepoMessage(response) => match response {
                RepoResponse::Channels(res) => {
                    match res {
                        Ok(channels) => {
                            self.channels = Some(channels.0);
                            self.channels_meta = Some(channels.1);
                        }
                        Err(e) => self.error = Some(e),
                    }
                    true
                }
                _ => false,
            },
            Message::SetShowAll(show_all) => {
                self.show_all = show_all;
                true
            }
            Message::SetActive(id, state) => {
                let mut meta = self
                    .channels_meta
                    .as_ref()
                    .unwrap()
                    .iter()
                    .find(|e| e.id == id)
                    .unwrap()
                    .clone();
                meta.active = state;
                self.repo.send(RepoRequest::SetChannelMeta(meta));
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
