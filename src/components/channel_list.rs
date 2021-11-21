use super::router::AppRoute;
use crate::agents::repo::{Repo, Request as RepoRequest, Response as RepoResponse};
use crate::objects::channel::Channel;
use anyhow::Error;
use yew::prelude::*;
use yew_router::prelude::RouterAnchor;

pub struct ChannelList {
    _link: ComponentLink<Self>,
    channels: Option<Vec<Channel>>,
    error: Option<Error>,
    _repo: Box<dyn Bridge<Repo>>,
}

pub enum Msg {
    RepoMessage(RepoResponse),
}

impl ChannelList {
    fn view_channel_list(&self) -> Html {
        match &self.channels {
            Some(c) => {
                html! {
                    <section class="section">
                        <div class="columns"><div class="column">
                            { c.iter().map(|i| html! { <RouterAnchor<AppRoute> classes={"navbar-item, card"} route={AppRoute::ItemsPage{channel_id: i.id}}>
                                <div class="card-content">
                                    <div class="media">
                                        <div class="media-left"><figure class="image is-64x64"><img src={i.image.clone()}/></figure></div>
                                        <div class="media-content"><p class="title">{&i.title}</p><p class="subtitle">{&i.description}</p></div>
                                    </div>
                                </div>
                            </RouterAnchor<AppRoute>> }).collect::<Html>() }
                        </div></div>
                    </section>
                }
            }
            None => html! { <p> {"no channels available"} </p> },
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
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let cb = link.callback(Msg::RepoMessage);
        let mut repo = Repo::bridge(cb);

        repo.send(RepoRequest::GetChannels);

        Self {
            _link: link,
            channels: None,
            error: None,
            _repo: repo,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::RepoMessage(response) => match response {
                RepoResponse::Channels(res) => {
                    match res {
                        Ok(channels) => {
                            self.channels = Some(channels.0);
                        }
                        Err(e) => self.error = Some(e),
                    }
                    true
                }
                _ => false,
            },
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
