use crate::agents::repo::{Repo, Request as RepoRequest, Response as RepoResponse};
use crate::objects::{channel::Channel, item::DownloadStatus, item::Item};
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
    ToggleDownload(Uuid),
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
                    <ion-list>
                        { items.iter().map(|i| {
                            let id = i.get_id();
                            html! { <ion-item><ion-label>
                                <ion-card-title>{&i.get_title()}</ion-card-title>
                                <ion-card-subtitle>{&i.get_date().format("%Y-%m-%d")}</ion-card-subtitle>
                                <ion-buttons>
                                    {match i.get_new() {
                                        true => html!(<ion-button color="primary" onclick={self.link.callback(move |_| Message::ToggleNew(id))}><ion-icon slot="start" name="star"/>{"new"}</ion-button>),
                                        false => html!(<ion-button onclick={self.link.callback(move |_| Message::ToggleNew(id))}><ion-icon slot="start" name="star-outline"/>{"new"}</ion-button>),
                                    }}
                                    {match i.get_download() {
                                        true => html!(<ion-button color="primary" onclick={self.link.callback(move |_| Message::ToggleDownload(id))}><ion-icon slot="start" name="cloud-download"/>{match &i.get_download_status() {
                                            DownloadStatus::Pending => "download pending",
                                            DownloadStatus::Ok(_) => "download ok",
                                            DownloadStatus::InProgress => "downloading",
                                            DownloadStatus::Error => "download error",
                                            _ => "download"
                                        }}</ion-button>),
                                        false => html!(<ion-button onclick={self.link.callback(move |_| Message::ToggleDownload(id))}><ion-icon slot="start" name="cloud-download"/>{"download"}</ion-button>)
                                    }}
                                </ion-buttons>
                        </ion-label></ion-item> }}).collect::<Html>() }
                    </ion-list>
                }
            }
            None => html! { <p> {"no items available"} </p> },
        }
    }

    fn view_pagination(&self) -> Html {
        match (&self.keys, &self.current_index) {
            (Some(keys), Some(current_index)) => {
                let mut ellopsis_drawn = false;

                html!(<ion-toolbar>
                {
                    if *current_index != 0 {
                        let new_index = current_index - 1;
                        html!(<ion-button slot="start" onclick={self.link.callback(move |_| Message::UpdateCurrentIndex(new_index))}><ion-icon slot="icon-only" name="chevron-back"/></ion-button>)
                    } else {
                        html!(<ion-button slot="start" disabled=true><ion-icon slot="icon-only" name="chevron-back"/></ion-button>)
                    }
                }
                {
                    if *current_index != keys.len()-1 {
                        let new_index = current_index + 1;
                        html!(<ion-button slot="end" onclick={self.link.callback(move |_| Message::UpdateCurrentIndex(new_index))}><ion-icon slot="icon-only" name="chevron-forward"/></ion-button>)
                    } else {
                        html!(<ion-button slot="end" disabled=true><ion-icon slot="icon-only" name="chevron-forward"/></ion-button>)
                    }
                }
                <ion-buttons>
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
                                html!(<ion-button disabled=true><ion-icon slot="icon-only" name="ellipsis-horizontal"/></ion-button>)
                            }
                        }
                    }
                }).collect::<Html>()}
                </ion-buttons>
              </ion-toolbar>)
            }
            _ => html!(),
        }
    }

    fn view_pagination_element(&self, key: &String, current_key: &String, index: usize) -> Html {
        if key == current_key {
            html!(<ion-button disabled=true fill="solid" color="primary">{key}</ion-button>)
        } else {
            html!(<ion-button onclick={self.link.callback(move |_| Message::UpdateCurrentIndex(index))}>{key}</ion-button>)
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
                    let mut item = items.iter().find(|i| i.get_id() == id).unwrap().clone();

                    item.set_new(!item.get_new());
                    self.repo.send(RepoRequest::UpdateItem(item));
                    false
                }
                None => false,
            },
            Message::ToggleDownload(id) => match &self.items {
                Some(items) => {
                    let mut item = items.iter().find(|i| i.get_id() == id).unwrap().clone();

                    item.set_download(!item.get_download());
                    self.repo.send(RepoRequest::UpdateItem(item));
                    false
                }
                None => false,
            },
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
                    res.sort_by(|a, b| b.get_date().partial_cmp(&a.get_date()).unwrap());

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
                            .find(|(_index, iter_item)| iter_item.get_id() == item.get_id())
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
