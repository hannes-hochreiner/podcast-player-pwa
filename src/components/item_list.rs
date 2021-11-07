use crate::agents::repo::{Repo, Request as RepoRequest};
use crate::objects::item::Item;
use anyhow::Error;
use uuid::Uuid;
use yew::{
    agent::Dispatcher,
    format::{Json, Nothing},
    prelude::*,
    services::fetch::{FetchService, FetchTask, Request, Response},
};

pub struct ItemList {
    _link: ComponentLink<Self>,
    fetch_task: Option<FetchTask>,
    items: Option<Vec<Item>>,
    error: Option<Error>,
    _repo: Dispatcher<Repo>,
}

pub enum Msg {
    ReceiveItems(Result<Vec<Item>, anyhow::Error>),
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
                            { items.iter().map(|i| html! { <div class="card">
                                <div class="card-content">
                                    <p class="title">{&i.title}</p>
                                </div>
                            </div> }).collect::<Html>() }
                        </div></div>
                    </section>
                }
            }
            None => html! { <p> {"no items available"} </p> },
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

impl Component for ItemList {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let request = Request::get(format!("/api/channels/{}/items", props.channel_id))
            .body(Nothing)
            .expect("Could not build request.");
        // 2. construct a callback
        let callback = link.callback(
            |response: Response<Json<Result<Vec<Item>, anyhow::Error>>>| {
                let Json(data) = response.into_body();
                Msg::ReceiveItems(data)
            },
        );
        // 3. pass the request and callback to the fetch service
        let task = FetchService::fetch(request, callback).expect("failed to start request");
        let mut disp = Repo::dispatcher();

        disp.send(RepoRequest::GetChannels);

        Self {
            _link: link,
            fetch_task: Some(task),
            items: None,
            error: None,
            _repo: disp,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ReceiveItems(res) => {
                match res {
                    Ok(c) => {
                        self.items = Some(c);
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
                { self.view_fetching() }
                { self.view_item_list() }
                { self.view_error() }
            </>
        }
    }
}
