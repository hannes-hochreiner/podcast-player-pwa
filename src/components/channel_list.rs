use super::router::AppRoute;
use crate::agents::{
    notifier,
    repo::{Repo, Request as RepoRequest, Response as RepoResponse},
};
use crate::components::icon::{Icon, IconStyle};
use crate::objects::{Channel, JsError};
use uuid::Uuid;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged, Dispatched, Dispatcher};
use yew_router::prelude::*;

pub struct ChannelList {
    channels: Option<Vec<Channel>>,
    error: Option<JsError>,
    repo: Box<dyn Bridge<Repo>>,
    show_all: bool,
    notifier: Dispatcher<notifier::Notifier>,
}

pub enum Message {
    RepoMessage(RepoResponse),
    SetShowAll(bool),
    SetActive(Uuid, bool),
}

impl ChannelList {
    fn view_channel_list(&self, ctx: &Context<Self>) -> Html {
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
                                                <li><a onclick={ctx.link().callback(move |_| Message::SetShowAll(true))}><span>{"All"}</span></a></li>
                                            </>
                                        }
                                    },
                                    true => {
                                        html! {
                                            <>
                                                <li><a onclick={ctx.link().callback(move |_| Message::SetShowAll(false))}><span>{"Active"}</span></a></li>
                                                <li class="is-active"><a><span>{"All"}</span></a></li>
                                            </>
                                        }
                                    },
                                }}
                            </ul>
                        </div>
                        <div class="columns"><div class="column">
                            { channels.iter().map(|channel| match self.show_all {
                                true => self.view_show_all_channel(ctx, channel),
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
        html! { <Link<AppRoute> classes={"navbar-item, card"} to={AppRoute::ItemsPage{channel_id: channel.val.id}}>
            <div class="card-content">
                <div class="media">
                    <div class="media-left"><figure class="image is-64x64"><img src={channel.val.image.clone()}/></figure></div>
                    <div class="media-content"><p class="title">{&channel.val.title}</p><p class="subtitle">{&channel.val.description}</p></div>
                </div>
            </div>
        </Link<AppRoute>> }
    }

    fn view_show_all_channel(&self, ctx: &Context<Self>, channel: &Channel) -> Html {
        let state = channel.meta.active;
        let channel_id = channel.val.id.clone();

        html! {
            <div class="card-content">
                <div class="media">
                    <div class="media-left"><figure class="image is-64x64"><img src={channel.val.image.clone()}/></figure></div>
                    <div class="media-content">
                        <p class="title">{&channel.val.title}</p><p class="subtitle">{&channel.val.description}</p>
                        {match state {
                            true => html!(<button class="button is-primary" onclick={ctx.link().callback(move |_| Message::SetActive(channel_id, false))}><Icon name="check" style={IconStyle::Outlined}/></button>),
                            false => html!(<button class="button" onclick={ctx.link().callback(move |_| Message::SetActive(channel_id, true))}><Icon name="check" style={IconStyle::Outlined}/></button>)
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

    fn process_update(&mut self, _ctx: &Context<Self>, msg: Message) -> Result<bool, JsError> {
        match msg {
            Message::RepoMessage(response) => match response {
                RepoResponse::Channels(res) => {
                    self.channels = Some(res);
                    Ok(true)
                }
                _ => Ok(false),
            },
            Message::SetShowAll(show_all) => {
                self.show_all = show_all;
                Ok(true)
            }
            Message::SetActive(id, state) => {
                let mut channel = self
                    .channels
                    .as_ref()
                    .ok_or("could not get channel reference")?
                    .iter()
                    .find(|e| e.val.id == id)
                    .ok_or("could not find channel")?
                    .clone();
                channel.meta.active = state;
                self.repo.send(RepoRequest::UpdateChannel(channel));
                Ok(false)
            }
        }
    }
}

impl Component for ChannelList {
    type Message = Message;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let cb = ctx.link().callback(Message::RepoMessage);
        let mut repo = Repo::bridge(cb);

        repo.send(RepoRequest::GetChannels);

        Self {
            channels: None,
            error: None,
            repo,
            show_all: false,
            notifier: notifier::Notifier::dispatcher(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match self.process_update(ctx, msg) {
            Ok(res) => res,
            Err(e) => {
                self.notifier.send(notifier::Request::NotifyError(e));
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
                { self.view_fetching() }
                { self.view_channel_list(ctx) }
                { self.view_error() }
            </>
        }
    }
}
