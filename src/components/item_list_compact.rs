use super::{Icon, IconStyle};
use crate::{
    agents::{notifier, repo},
    objects::JsError,
};
use podcast_player_common::{item_meta::DownloadStatus, Item};
use uuid::Uuid;
use yew::{prelude::*, Component, Properties};
use yew_agent::{Dispatched, Dispatcher};

pub struct ItemListCompact {
    repo: Dispatcher<repo::Repo>,
    show_content: bool,
    notifier: Dispatcher<notifier::Notifier>,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub items: Vec<Item>,
    pub show_details: bool,
    pub on_selected: Callback<Item>,
}

pub enum Message {
    ToggleShowContent,
    ToggleNew(Uuid),
    ToggleDownload(Uuid),
}

impl ItemListCompact {
    fn process_update(&mut self, ctx: &Context<Self>, msg: Message) -> Result<bool, JsError> {
        match msg {
            Message::ToggleShowContent => {
                self.show_content = !self.show_content;
                Ok(true)
            }
            Message::ToggleNew(item_id) => {
                let mut item = ctx
                    .props()
                    .items
                    .iter()
                    .find(|i| i.get_id() == item_id)
                    .ok_or("item not found")?
                    .clone();

                item.set_new(!item.get_new());
                self.repo.send(repo::Request::UpdateItem(item));
                Ok(false)
            }
            Message::ToggleDownload(item_id) => {
                let mut item = ctx
                    .props()
                    .items
                    .iter()
                    .find(|i| i.get_id() == item_id)
                    .ok_or("item not found")?
                    .clone();

                match item.get_download_status() {
                    DownloadStatus::Error | DownloadStatus::Ok => {
                        self.repo.send(repo::Request::DeleteEnclosure(item));
                    }
                    DownloadStatus::Pending => {
                        item.set_download_status(DownloadStatus::NotRequested);
                        self.repo.send(repo::Request::UpdateItem(item));
                    }
                    DownloadStatus::InProgress => {}
                    DownloadStatus::NotRequested => {
                        item.set_download_status(DownloadStatus::Pending);
                        self.repo.send(repo::Request::UpdateItem(item));
                    }
                }

                Ok(false)
            }
        }
    }

    fn view_card_content(&self, ctx: &yew::Context<Self>, item: &Item) -> Html {
        let id = item.get_id();

        html! {<div class="card-content">
            <div class="field is-grouped is-grouped-multiline">
                <div class="control">
                    <div class="tags">
                        <span class="tag">{item.get_date().format("%Y-%m-%d").to_string()}</span>
                    </div>
                </div>
                <div class="control">
                    <div class="tags has-addons">
                        <span class="tag">{format!("{}x", item.get_play_count())}</span>
                        <span class="tag is-primary">{"played"}</span>
                    </div>
                </div>
            </div>
            <p class="buttons">
                {match item.get_new() {
                    true => html!(<button class="button is-primary" onclick={ctx.link().callback(move |_| Message::ToggleNew(id))}><Icon name="star" style={IconStyle::Filled}/><span>{"new"}</span></button>),
                    false => html!(<button class="button" onclick={ctx.link().callback(move |_| Message::ToggleNew(id))}><Icon name="star_outline" style={IconStyle::Filled}/><span>{"new"}</span></button>),
                }}
                <button class="button is-primary" onclick={ctx.link().callback(move |_| Message::ToggleDownload(id))}>{match item.get_download_status() {
                    DownloadStatus::Pending => html!{<><Icon name="cloud_queue" style={IconStyle::Filled}/><span>{"download pending"}</span></>},
                    DownloadStatus::Ok => html!{<><Icon name="cloud_done" style={IconStyle::Filled}/><span>{"download ok"}</span></>},
                    DownloadStatus::InProgress => html!{<><Icon name="cloud_sync" style={IconStyle::Filled}/><span>{"downloading"}</span></>},
                    DownloadStatus::Error => html!{<><Icon name="cloud_off" style={IconStyle::Filled}/><span>{"download error"}</span></>},
                    _ => html!{<span>{"download"}</span>}
                }}</button>
            </p>
        </div>}
    }
}

impl Component for ItemListCompact {
    type Message = Message;
    type Properties = Props;

    fn create(ctx: &yew::Context<Self>) -> Self {
        Self {
            repo: repo::Repo::dispatcher(),
            show_content: ctx.props().show_details,
            notifier: notifier::Notifier::dispatcher(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match self.process_update(ctx, msg) {
            Ok(update) => update,
            Err(e) => {
                self.notifier.send(notifier::Request::NotifyError(e));
                true
            }
        }
    }

    fn view(&self, ctx: &yew::Context<Self>) -> Html {
        html! {
            <div class="columns is-multiline">
                { ctx.props().items.iter().map(|i| {
                    let onselect_item = i.clone();
                    let on_selected = ctx.props().on_selected.clone();
                    html! { <div class="column is-one-quarter"><div class="card">
                    <header class="card-header">
                        <p class="card-header-title" onclick={move |_| on_selected.emit(onselect_item.clone())}>{&i.get_title()}</p>
                        <button class="card-header-icon" onclick={ctx.link().callback(|_| Message::ToggleShowContent)}><Icon name={match self.show_content { true => "expand_less", false => "expand_more"}} style={IconStyle::Outlined}/></button>
                    </header>
                    {if self.show_content {
                        self.view_card_content(ctx, i)
                    } else {
                        html!{}
                    }}
                </div></div> }}).collect::<Html>() }
            </div>
        }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        self.show_content = ctx.props().show_details;

        true
    }
}
