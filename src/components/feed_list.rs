use crate::agents::repo::{Repo, Request as RepoRequest, Response as RepoResponse};
use crate::objects::feed::Feed;
use anyhow::Error;
use yew::prelude::*;

pub struct FeedList {
    link: ComponentLink<Self>,
    feeds: Option<Vec<Feed>>,
    error: Option<Error>,
    repo: Box<dyn Bridge<Repo>>,
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
                        <div class="columns"><div class="column">
                            { feeds.iter().map(|feed| self.view_show_feed(feed)).collect::<Html>() }
                        </div></div>
                    </section>
                }
            }
            _ => html! { <p> {"no feeds available"} </p> },
        }
    }

    fn view_show_feed(&self, feed: &Feed) -> Html {
        html! {
            <div class="card-content">
                <div class="media">
                    <div class="media-content">
                        <p class="title">{&feed.val.url}</p>
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

impl Component for FeedList {
    type Message = Message;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let cb = link.callback(Message::RepoMessage);
        let mut repo = Repo::bridge(cb);

        repo.send(RepoRequest::GetFeeds);

        Self {
            link: link,
            feeds: None,
            error: None,
            repo,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::RepoMessage(response) => match response {
                RepoResponse::Feeds(res) => {
                    self.feeds = Some(res);
                    true
                }
                RepoResponse::Error(e) => {
                    log::info!("feed list error: {}", e);
                    false
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
                { self.view_feed_list() }
                { self.view_error() }
            </>
        }
    }
}
