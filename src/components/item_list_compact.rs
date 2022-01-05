use super::{Icon, IconStyle};
use crate::agents::repo;
use podcast_player_common::{item_meta::DownloadStatus, Item};
use yew::{prelude::*, Component, Properties};
use yew_agent::{Dispatched, Dispatcher};

pub struct ItemListCompact {
    repo: Dispatcher<repo::Repo>,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub items: Vec<Item>,
    pub on_selected: Callback<Item>,
}

pub enum Message {
    RemoveDownload(Item),
}

impl Component for ItemListCompact {
    type Message = Message;
    type Properties = Props;

    fn create(_ctx: &yew::Context<Self>) -> Self {
        Self {
            repo: repo::Repo::dispatcher(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::RemoveDownload(item) => self.repo.send(repo::Request::DeleteEnclosure(item)),
        }

        false
    }

    fn view(&self, ctx: &yew::Context<Self>) -> Html {
        html! {
            <div class="columns is-multiline">
                { ctx.props().items.iter().map(|i| {
                    let onselect_item = i.clone();
                    let delete_item = i.clone();
                    let on_selected = ctx.props().on_selected.clone();
                    html! { <div class="column is-one-quarter"><div class="card">
                    <div class="card-content" onclick={move |_| on_selected.emit(onselect_item.clone())}>
                        <p class="has-text-weight-bold">{&i.get_title()}</p>
                        <div class="field is-grouped is-grouped-multiline">
                            <div class="control">
                                <div class="tags">
                                    <span class="tag">{&i.get_date().format("%Y-%m-%d").to_string()}</span>
                                </div>
                            </div>
                            <div class="control">
                                <div class="tags has-addons">
                                    <span class="tag">{format!("{}x", &i.get_play_count())}</span>
                                    <span class="tag is-primary">{"played"}</span>
                                </div>
                            </div>
                            <div class="control">{match &i.get_download_status() {
                                DownloadStatus::Pending => html!{<div class="tags has-addons"><span class="tag"><Icon name="cloud_queue" style={IconStyle::Filled}/></span><span class="tag is-primary">{"download pending"}</span></div>},
                                DownloadStatus::Ok(_) => html!{<div class="tags has-addons"><span class="tag"><Icon name="cloud_done" style={IconStyle::Filled}/></span><span class="tag is-primary">{"download ok"}</span></div>},
                                DownloadStatus::InProgress => html!{<div class="tags has-addons"><span class="tag"><Icon name="cloud_sync" style={IconStyle::Filled}/></span><span class="tag is-primary">{"downloading"}</span></div>},
                                DownloadStatus::Error => html!{<div class="tags has-addons"><span class="tag"><Icon name="cloud_off" style={IconStyle::Filled}/></span><span class="tag is-primary">{"download error"}</span></div>},
                                _ => html!{<div class="tags"><span class="tag">{"download"}</span></div>}
                            }}
                            </div>
                        </div>
                    </div>
                    <footer class="card-footer">
                        <a href="#" class="card-footer-item">{"mark as new"}</a>
                        <a class="card-footer-item" onclick={ctx.link().callback(move |_| Message::RemoveDownload(delete_item.clone()))}>{"remove download"}</a>
                    </footer>
                </div></div> }}).collect::<Html>() }
            </div>
        }
    }

    fn changed(&mut self, _ctx: &Context<Self>) -> bool {
        true
    }
}
