use std::cmp::Ordering;

use crate::agents::{
    notifier,
    repo::{Repo, Request as RepoRequest, Response as RepoResponse},
};
use crate::components::icon::{Icon, IconStyle};
use crate::objects::{Channel, DownloadStatus, Item, JsError};
use uuid::Uuid;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged, Dispatched, Dispatcher};

pub struct ItemList {
    items: Option<Vec<Item>>,
    error: Option<JsError>,
    repo: Box<dyn Bridge<Repo>>,
    channel: Option<Channel>,
    keys: Option<Vec<String>>,
    current_index: Option<usize>,
    notifier: Dispatcher<notifier::Notifier>,
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
    fn view_item_list(&self, ctx: &Context<Self>) -> Html {
        match &self.items {
            Some(items) => {
                html! {
                    <section class="section">
                        <div class="columns"><div class="column">
                            { items.iter().map(|i| {
                                let id = i.get_id();
                                html! { <div class="card">
                                <div class="card-content">
                                    <p class="title">{&i.get_title()}</p>
                                    <p class="subtitle">{&i.get_date().format("%Y-%m-%d")}{&i.get_id()}</p>
                                    <p class="buttons">
                                        {match i.get_new() {
                                            true => html!(<button class="button is-primary" onclick={ctx.link().callback(move |_| Message::ToggleNew(id))}><Icon name="star" style={IconStyle::Filled}/><span>{"new"}</span></button>),
                                            false => html!(<button class="button" onclick={ctx.link().callback(move |_| Message::ToggleNew(id))}><Icon name="star_outline" style={IconStyle::Filled}/><span>{"new"}</span></button>),
                                        }}
                                        {match i.get_download() {
                                            true => html!(<button class="button is-primary" onclick={ctx.link().callback(move |_| Message::ToggleDownload(id))}>{match &i.get_download_status() {
                                                DownloadStatus::Pending => html!{<><Icon name="cloud_queue" style={IconStyle::Filled}/><span>{"download pending"}</span></>},
                                                DownloadStatus::Ok(_) => html!{<><Icon name="cloud_done" style={IconStyle::Filled}/><span>{"download ok"}</span></>},
                                                DownloadStatus::InProgress => html!{<><Icon name="cloud_sync" style={IconStyle::Filled}/><span>{"downloading"}</span></>},
                                                DownloadStatus::Error => html!{<><Icon name="cloud_off" style={IconStyle::Filled}/><span>{"download error"}</span></>},
                                                _ => html!{<span>{"download"}</span>}
                                            }}</button>),
                                            false => html!(<button class="button" onclick={ctx.link().callback(move |_| Message::ToggleDownload(id))}><Icon name="cloud_download" style={IconStyle::Outlined}/><span>{"download"}</span></button>)
                                        }}
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

    fn view_pagination(&self, ctx: &Context<Self>) -> Html {
        match (&self.keys, &self.current_index) {
            (Some(keys), Some(current_index)) => {
                let mut ellopsis_drawn = false;

                html!(<nav class="pagination is-centered" role="navigation" aria-label="pagination">
                {
                    if *current_index != 0 {
                        let new_index = current_index - 1;
                        html!(<a class="pagination-previous" onclick={ctx.link().callback(move |_| Message::UpdateCurrentIndex(new_index))}>{"<"}</a>)
                    } else {
                        html!()
                    }
                }
                {
                    if *current_index != keys.len()-1 {
                        let new_index = current_index + 1;
                        html!(<a class="pagination-next" onclick={ctx.link().callback(move |_| Message::UpdateCurrentIndex(new_index))}>{">"}</a>)
                    } else {
                        html!()
                    }
                }
                <ul class="pagination-list">
                { keys.iter().enumerate().map(|(idx, key)| {
                    if idx == 0 {
                        ellopsis_drawn = false;
                        self.view_pagination_element(ctx,key, &keys[*current_index], idx)
                    }
                    else if idx == keys.len() -1 {
                        ellopsis_drawn = false;
                        self.view_pagination_element(ctx, key, &keys[*current_index], idx)
                    } else if idx == *current_index {
                        ellopsis_drawn = false;
                        self.view_pagination_element(ctx,key, &keys[*current_index], idx)
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

    fn view_pagination_element(
        &self,
        ctx: &Context<Self>,
        key: &String,
        current_key: &String,
        index: usize,
    ) -> Html {
        if key == current_key {
            html!(<li><a class="pagination-link is-current">{key}</a></li>)
        } else {
            html!(<li><a class="pagination-link" onclick={ctx.link().callback(move |_| Message::UpdateCurrentIndex(index))}>{key}</a></li>)
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

    fn process_update(&mut self, ctx: &Context<Self>, msg: Message) -> Result<bool, JsError> {
        match msg {
            Message::UpdateCurrentIndex(idx) => {
                if let Some(channel) = &self.channel {
                    self.current_index = Some(idx);
                    self.items = None;
                    self.repo.send(RepoRequest::GetItemsByChannelIdYearMonth(
                        channel.val.id,
                        self.keys.as_ref().ok_or("could not get reference")?[idx].clone(),
                    ));
                }
                Ok(false)
            }
            Message::ToggleNew(id) => match &self.items {
                Some(items) => {
                    let mut item = items
                        .iter()
                        .find(|i| i.get_id() == id)
                        .ok_or("could not find item")?
                        .clone();

                    item.set_new(!item.get_new());
                    self.repo.send(RepoRequest::UpdateItem(item));
                    Ok(false)
                }
                None => Ok(false),
            },
            Message::ToggleDownload(id) => match &self.items {
                Some(items) => {
                    let mut item = items
                        .iter()
                        .find(|i| i.get_id() == id)
                        .ok_or("could not find item")?
                        .clone();

                    item.set_download(!item.get_download());
                    self.repo.send(RepoRequest::UpdateItem(item));
                    Ok(false)
                }
                None => Ok(false),
            },
            Message::RepoMessage(resp) => match resp {
                RepoResponse::Channels(res) => {
                    // let channel_id = channel.val.id.clone();
                    let channel = res
                        .iter()
                        .find(|e| e.val.id == ctx.props().channel_id)
                        .ok_or("could not find channel")?
                        .clone();
                    let mut keys: Vec<String> = channel
                        .keys
                        .year_month_keys
                        .iter()
                        .map(|e| e.clone())
                        .collect();
                    // unwrap is safe, as a default is provided
                    keys.sort_by(|a, b| b.partial_cmp(a).or(Some(Ordering::Equal)).unwrap());

                    self.current_index = Some(0);
                    self.keys = Some(keys);

                    self.channel = Some(channel);
                    self.repo.send(RepoRequest::GetItemsByChannelIdYearMonth(
                        ctx.props().channel_id,
                        self.keys.as_ref().ok_or("could not get reference")?[0].clone(),
                    ));
                    Ok(false)
                }
                RepoResponse::Items(mut res) => {
                    // unwrap is safe, as a default is provided
                    res.sort_by(|a, b| {
                        b.get_date()
                            .partial_cmp(&a.get_date())
                            .or(Some(Ordering::Equal))
                            .unwrap()
                    });

                    self.items = Some(res);
                    Ok(true)
                }
                RepoResponse::Error(e) => {
                    self.error = Some(e);
                    Ok(true)
                }
                RepoResponse::Item(item) => match &mut self.items {
                    Some(items) => {
                        let index = items
                            .iter()
                            .enumerate()
                            .find(|(_index, iter_item)| iter_item.get_id() == item.get_id())
                            .ok_or("could not find item")?
                            .0;
                        items[index] = item;
                        Ok(true)
                    }
                    None => Ok(false),
                },
                _ => Ok(false),
            },
        }
    }
}

impl Component for ItemList {
    type Message = Message;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let cb = ctx.link().callback(Message::RepoMessage);
        let mut repo = Repo::bridge(cb);

        repo.send(RepoRequest::GetChannels);

        Self {
            items: None,
            error: None,
            channel: None,
            repo,
            current_index: None,
            keys: None,
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
                { self.view_pagination(ctx) }
                { self.view_fetching() }
                { self.view_item_list(ctx) }
                { self.view_error() }
            </>
        }
    }
}
