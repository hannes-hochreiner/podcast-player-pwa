use crate::agents::{
    notifier,
    repo::{Repo, Request as RepoRequest, Response as RepoResponse},
};
use crate::objects::{Feed, JsError};
use yew::prelude::*;
use yew_agent::{Bridge, Bridged, Dispatched, Dispatcher};

pub struct FeedNew {
    feeds: Option<Vec<Feed>>,
    error: Option<JsError>,
    repo: Box<dyn Bridge<Repo>>,
    input_ref: yew::NodeRef,
    notifier: Dispatcher<notifier::Notifier>,
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

    fn process_update(&mut self, _ctx: &Context<Self>, msg: Message) -> Result<bool, JsError> {
        match msg {
            Message::RepoMessage(response) => match response {
                RepoResponse::Feeds(res) => {
                    self.feeds = Some(res);
                    Ok(true)
                }
                RepoResponse::Error(e) => {
                    log::info!("feed list error: {}", e);
                    Ok(false)
                }
                _ => Ok(false),
            },
            Message::Submit => {
                let elem = self
                    .input_ref
                    .cast::<web_sys::HtmlInputElement>()
                    .ok_or("could not get input element")?;

                self.repo.send(RepoRequest::AddFeed(elem.value()));
                elem.set_value("");
                Ok(true)
            }
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
                { self.view_channel_list(ctx) }
                { self.view_error() }
            </>
        }
    }
}
