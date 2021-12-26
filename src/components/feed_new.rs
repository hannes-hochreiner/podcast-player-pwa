use crate::agents::repo::{Repo, Request as RepoRequest, Response as RepoResponse};
use crate::objects::Feed;
use anyhow::Error;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

pub struct FeedNew {
    feeds: Option<Vec<Feed>>,
    error: Option<Error>,
    repo: Box<dyn Bridge<Repo>>,
    input_ref: yew::NodeRef,
}

pub enum Message {
    RepoMessage(RepoResponse),
    Submit,
}

impl FeedNew {
    fn view_channel_list(&self, ctx: &Context<Self>) -> Html {
        html! {
            <section class="section">
                <div class="columns">
                    <div class="column"><input class="input" ref={self.input_ref.clone()} type="text" placeholder="feed url"/></div>
                    <div class="column"><button class="button" onclick={ctx.link().callback(|_| Message::Submit)}>{"submit"}</button></div>
                </div>
            </section>
        }
    }

    fn view_error(&self) -> Html {
        match &self.error {
            Some(e) => html! { <p>{e}</p> },
            None => html! {},
        }
    }
}

impl Component for FeedNew {
    type Message = Message;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let cb = ctx.link().callback(Message::RepoMessage);
        let mut repo = Repo::bridge(cb);

        repo.send(RepoRequest::GetFeeds);

        Self {
            feeds: None,
            error: None,
            repo,
            input_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
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
            Message::Submit => {
                let elem = self.input_ref.cast::<web_sys::HtmlInputElement>().unwrap();

                self.repo.send(RepoRequest::AddFeed(elem.value()));
                elem.set_value("");
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
                { self.view_channel_list(ctx) }
                { self.view_error() }
            </>
        }
    }
}
