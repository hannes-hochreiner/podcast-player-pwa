use crate::agents::repo::{Repo, Request as RepoRequest, Response as RepoResponse};
use crate::objects::item::Item;
use anyhow::Error;
use uuid::Uuid;
use yew::prelude::*;

pub struct ItemList {
    _link: ComponentLink<Self>,
    items: Option<Vec<Item>>,
    error: Option<Error>,
    repo: Box<dyn Bridge<Repo>>,
}

pub enum Message {
    RepoMessage(RepoResponse),
    Download(Uuid),
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub channel_id: Uuid,
}

impl ItemList {
    fn view_item_list(&self) -> Html {
        match &self.items {
            Some(items) => {
                html! {
                    <section class="section">
                        <div class="columns"><div class="column">
                            { items.iter().map(|i| {
                                let id = i.val.id;
                                html! { <div class="card">
                                <div class="card-content">
                                    <p class="title">{&i.val.title}</p>
                                    <p class="subtitle">{&i.val.date}</p>
                                    <button class="button" onclick={self._link.callback(move |_| Message::Download(id))}>{"download"}</button>
                                </div>
                            </div> }}).collect::<Html>() }
                        </div></div>
                    </section>
                }
            }
            None => html! { <p> {"no items available"} </p> },
        }
    }

    fn view_fetching(&self) -> Html {
        if self.items.is_none() {
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

impl Component for ItemList {
    type Message = Message;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let cb = link.callback(Message::RepoMessage);
        let mut repo = Repo::bridge(cb);

        // repo.send(RepoRequest::GetItems);
        repo.send(RepoRequest::GetItemsByChannelIdYearMonth(
            props.channel_id,
            "2021-10".to_string(),
        ));

        Self {
            _link: link,
            items: None,
            error: None,
            repo,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::Download(id) => {
                self.repo.send(RepoRequest::DownloadEnclosure(id));
                false
            }
            Message::RepoMessage(resp) => match resp {
                RepoResponse::Items(res) => {
                    match res {
                        Ok(c) => {
                            self.items = Some(c);
                        }
                        Err(e) => {
                            self.error = Some(e);
                        }
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
                { self.view_item_list() }
                { self.view_error() }
            </>
        }
    }
}
