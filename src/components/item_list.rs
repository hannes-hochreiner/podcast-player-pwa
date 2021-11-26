use std::collections::HashMap;

use crate::agents::repo::{Repo, Request as RepoRequest, Response as RepoResponse};
use crate::objects::{channel::Channel, item::Item};
use anyhow::Error;
use uuid::Uuid;
use yew::prelude::*;

pub struct ItemList {
    link: ComponentLink<Self>,
    items: Option<Vec<Item>>,
    error: Option<Error>,
    repo: Box<dyn Bridge<Repo>>,
    channel: Option<Channel>,
    channel_id: Uuid,
    keys: Option<Vec<String>>,
    current_index: Option<usize>,
}

pub enum Message {
    RepoMessage(RepoResponse),
    Download(Uuid),
    UpdateCurrentIndex(usize),
    ToggleNew(Uuid),
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
                                    <p class="buttons">
                                        {match i.meta.new {
                                            true => html!(<button class="button is-primary" onclick={self.link.callback(move |_| Message::ToggleNew(id))}><span class="icon"><ion-icon size="large" name="star"/></span><span>{"new"}</span></button>),
                                            false => html!(<button class="button" onclick={self.link.callback(move |_| Message::ToggleNew(id))}><span class="icon"><ion-icon size="large" name="star-outline"/></span><span>{"new"}</span></button>),
                                        }}
                                        <button class="button" onclick={self.link.callback(move |_| Message::Download(id))}><span class="icon"><ion-icon size="large" name="cloud-download"/></span><span>{"download"}</span></button>
                                    </p>
                                </div>
                            </div> }}).collect::<Html>() }
                        </div></div>
                    </section>
                }
            }
            None => html! { <p> {"no items available"} </p> },
        }
    }

    fn view_pagination(&self) -> Html {
        match (&self.keys, &self.current_index) {
            (Some(keys), Some(current_index)) => {
                let mut ellopsis_drawn = false;

                html!(<nav class="pagination is-centered" role="navigation" aria-label="pagination">
                {
                    if *current_index != 0 {
                        let new_index = current_index - 1;
                        html!(<a class="pagination-previous" onclick={self.link.callback(move |_| Message::UpdateCurrentIndex(new_index))}>{"<"}</a>)
                    } else {
                        html!()
                    }
                }
                {
                    if *current_index != keys.len()-1 {
                        let new_index = current_index + 1;
                        html!(<a class="pagination-next" onclick={self.link.callback(move |_| Message::UpdateCurrentIndex(new_index))}>{">"}</a>)
                    } else {
                        html!()
                    }
                }
                <ul class="pagination-list">
                { keys.iter().enumerate().map(|(idx, key)| {
                    if idx == 0 {
                        ellopsis_drawn = false;
                        self.view_pagination_element(key, &keys[*current_index], idx)
                    }
                    else if idx == keys.len() -1 {
                        ellopsis_drawn = false;
                        self.view_pagination_element(key, &keys[*current_index], idx)
                    } else if idx == *current_index {
                        ellopsis_drawn = false;
                        self.view_pagination_element(key, &keys[*current_index], idx)
                    } else {
                        match ellopsis_drawn {
                            true =>  html!(),
                            false => {
                                ellopsis_drawn = true;
                                html!(<li><span class="pagination-ellipsis">{"..."}</span></li>)
                            }
                        }
                    }
                }).collect::<Html>()}
                </ul>
              </nav>)
            }
            _ => html!(),
        }
    }

    fn view_pagination_element(&self, key: &String, current_key: &String, index: usize) -> Html {
        if key == current_key {
            html!(<li><a class="pagination-link is-current">{key}</a></li>)
        } else {
            html!(<li><a class="pagination-link" onclick={self.link.callback(move |_| Message::UpdateCurrentIndex(index))}>{key}</a></li>)
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

        repo.send(RepoRequest::GetChannels);

        Self {
            link,
            items: None,
            error: None,
            channel: None,
            repo,
            channel_id: props.channel_id,
            current_index: None,
            keys: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::UpdateCurrentIndex(idx) => {
                self.current_index = Some(idx);
                self.items = None;
                self.repo.send(RepoRequest::GetItemsByChannelIdYearMonth(
                    self.channel_id,
                    self.keys.as_ref().unwrap()[idx].clone(),
                ));
                false
            }
            Message::ToggleNew(id) => match &self.items {
                Some(items) => {
                    let mut item = items.iter().find(|i| i.val.id == id).unwrap().clone();

                    item.meta.new = !item.meta.new;
                    self.repo.send(RepoRequest::UpdateItem(item));
                    false
                }
                None => false,
            },
            Message::Download(id) => {
                // self.repo.send(RepoRequest::DownloadEnclosure(id));
                false
            }
            Message::RepoMessage(resp) => match resp {
                RepoResponse::Channels(res) => {
                    let channel = res
                        .iter()
                        .find(|e| e.val.id == self.channel_id)
                        .unwrap()
                        .clone();
                    let mut keys: Vec<String> = channel
                        .keys
                        .year_month_keys
                        .iter()
                        .map(|e| e.clone())
                        .collect();
                    keys.sort_by(|a, b| b.partial_cmp(a).unwrap());

                    self.current_index = Some(0);
                    self.keys = Some(keys);

                    self.channel = Some(channel);
                    self.repo.send(RepoRequest::GetItemsByChannelIdYearMonth(
                        self.channel_id,
                        self.keys.as_ref().unwrap()[0].clone(),
                    ));
                    false
                }
                RepoResponse::Items(mut res) => {
                    res.sort_by(|a, b| b.val.date.partial_cmp(&a.val.date).unwrap());

                    self.items = Some(res);
                    true
                }
                RepoResponse::Error(e) => {
                    self.error = Some(e);
                    true
                }
                RepoResponse::Item(item) => match &mut self.items {
                    Some(items) => {
                        let index = items
                            .iter()
                            .enumerate()
                            .find(|(_index, iter_item)| iter_item.val.id == item.val.id)
                            .unwrap()
                            .0;
                        items[index] = item;
                        true
                    }
                    None => false,
                },
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
                { self.view_pagination() }
                { self.view_fetching() }
                { self.view_item_list() }
                { self.view_error() }
            </>
        }
    }
}
