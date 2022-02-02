use crate::agents::repo::{Repo, Request as RepoRequest, Response as RepoResponse};
use crate::objects::{FeedVal, JsError};
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

pub struct FeedList {
    feeds: Option<Vec<FeedVal>>,
    error: Option<JsError>,
    _repo: Box<dyn Bridge<Repo>>,
}

pub enum Message {
    RepoMessage(RepoResponse),
}

impl FeedList {
    fn view_feed_list(&self) -> Html {
        match &self.feeds {
            Some(feeds) => {
                html! {
                    <section class="section">
                        <div class="columns is-multiline"><div class="column is-one-quarter">
                            { feeds.iter().map(|feed| self.view_show_feed(feed)).collect::<Html>() }
                        </div></div>
                    </section>
                }
            }
            _ => html! { <p> {"no feeds available"} </p> },
        }
    }

    fn view_show_feed(&self, feed: &FeedVal) -> Html {
        html! {
            <div class="card">
                <header class="card-header">
                    <p class="card-header-title">{&feed.title}</p>
                </header>
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

impl Component for FeedList {
    type Message = Message;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let cb = ctx.link().callback(Message::RepoMessage);
        let mut repo = Repo::bridge(cb);

        repo.send(RepoRequest::GetFeeds);

        Self {
            feeds: None,
            error: None,
            _repo: repo,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::RepoMessage(response) => match response {
                RepoResponse::Feeds(res) => {
                    self.feeds = Some(res);
                    true
                }
                _ => false,
            },
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <>
                { self.view_fetching() }
                { self.view_feed_list() }
                { self.view_error() }
            </>
        }
    }
}
