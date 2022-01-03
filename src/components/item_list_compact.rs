use super::{Icon, IconStyle};
use podcast_player_common::{item_meta::DownloadStatus, Item};
use yew::{prelude::*, Component, Properties};

pub struct ItemListCompact {}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub items: Vec<Item>,
    pub on_selected: Callback<Item>,
}

pub enum Message {}

impl Component for ItemListCompact {
    type Message = Message;
    type Properties = Props;

    fn create(_ctx: &yew::Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &yew::Context<Self>) -> Html {
        html! {
            <div class="columns"><div class="column">
                { ctx.props().items.iter().map(|i| {
                    let new_source = i.clone();
                    let on_selected = ctx.props().on_selected.clone();
                    html! { <div class="card" onclick={move |_| on_selected.emit(new_source.clone())}>
                    <div class="card-content">
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
                </div> }}).collect::<Html>() }
            </div></div>
        }
    }
}
