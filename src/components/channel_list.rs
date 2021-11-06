use crate::agents::repo::{Repo, Request as RepoRequest};
use crate::objects::channel::Channel;
use anyhow::Error;
use yew::{
    agent::Dispatcher,
    format::{Json, Nothing},
    prelude::*,
    services::fetch::{FetchService, FetchTask, Request, Response},
};

pub struct ChannelList {
    link: ComponentLink<Self>,
    fetch_task: Option<FetchTask>,
    channels: Option<Vec<Channel>>,
    error: Option<Error>,
    repo: Dispatcher<Repo>,
}

pub enum Msg {
    ReceiveChannels(Result<Vec<Channel>, anyhow::Error>),
}

impl ChannelList {
    fn view_channel_list(&self) -> Html {
        match &self.channels {
            Some(c) => {
                html! {<div>
                    <section class="section">
                        <div class="columns"><div class="column">
                            { c.iter().map(|i| html! { <div class="card">
                                <div class="card-content">
                                    <div class="media">
                                        <div class="media-left"><figure class="image is-64x64"><img src={i.image.clone()}/></figure></div>
                                        <div class="media-content"><p class="title">{&i.title}</p></div>
                                    </div>
                                </div>
                            </div> }).collect::<Html>() }
                    </div></div></section>
                </div>}
            }
            None => html! { <p> {"no channels available"} </p> },
        }
    }

    fn view_fetching(&self) -> Html {
        if self.fetch_task.is_some() {
            html! { <p>{ "Fetching data..." }</p> }
        } else {
            html! {}
        }
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
        let request = Request::get("/api/channels")
            .body(Nothing)
            .expect("Could not build request.");
        // 2. construct a callback
        let callback = link.callback(
            |response: Response<Json<Result<Vec<Channel>, anyhow::Error>>>| {
                let Json(data) = response.into_body();
                Msg::ReceiveChannels(data)
            },
        );
        // 3. pass the request and callback to the fetch service
        let task = FetchService::fetch(request, callback).expect("failed to start request");
        let mut disp = Repo::dispatcher();

        disp.send(RepoRequest::GetChannels);

        Self {
            link,
            fetch_task: Some(task),
            channels: None,
            error: None,
            repo: disp,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ReceiveChannels(res) => {
                match res {
                    Ok(c) => {
                        self.channels = Some(c);
                    }
                    Err(e) => {
                        self.error = Some(e);
                    }
                }

                self.fetch_task = None;
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
                <nav class="navbar is-primary" role="navigation">
                    <div class="navbar-brand"><div class="navbar-item title">{"Podcast Player"}</div></div>
                </nav>
                { self.view_fetching() }
                { self.view_channel_list() }
                { self.view_error() }
            </>
        }
    }
}
