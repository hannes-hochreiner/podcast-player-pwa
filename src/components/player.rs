use crate::{
    // agents::{notifier, player, repo},
    components::{
        icon::{Icon, IconStyle},
        item_list_compact::ItemListCompact,
        Range,
    },
    objects::{DownloadStatus, Item, JsError},
};
use podcast_player_common::Channel;
use std::cmp::Ordering;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged, Dispatched, Dispatcher};

pub enum Tab {
    Unplayed,
    Downloaded,
}

pub struct Player {
    // _repo: Box<dyn Bridge<repo::Repo>>,
    // player: Box<dyn Bridge<player::Player>>,
    items: Option<Vec<Item>>,
    source: Option<(Item, Channel)>,
    duration: Option<f64>,
    // notifier: Dispatcher<notifier::Notifier>,
    is_playing: bool,
    show_sliders: bool,
    tab: Tab,
    status_obtained: bool,
}
pub enum Message {
    // RepoMessage(repo::Response),
    // PlayerMessage(player::Response),
    SetSource(Option<Item>),
    Play,
    Pause,
    TimeChange(String),
    VolumeChange(String),
    PlaybackRateChange(String),
    ToggleShowSliders,
    SwitchTab(Tab),
}

impl Player {
    fn view_tabs(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="tabs is-centered is-toggle">
                {match self.tab {
                    Tab::Unplayed => html!{
                        <ul>
                            <li class="is-active"><a>{"unplayed"}</a></li>
                            <li><a onclick={ctx.link().callback(|_| Message::SwitchTab(Tab::Downloaded))}>{"downloaded"}</a></li>
                        </ul>
                    },
                    Tab::Downloaded => html!{
                        <ul>
                            <li><a onclick={ctx.link().callback(|_| Message::SwitchTab(Tab::Unplayed))}>{"unplayed"}</a></li>
                            <li class="is-active"><a>{"downloaded"}</a></li>
                        </ul>
                    }
                }}
            </div>
        }
    }

    fn view_item_list(&self, ctx: &Context<Self>) -> Html {
        match &self.items {
            Some(items) => {
                let filtered_items: Vec<Item> = items
                    .iter()
                    .filter(|i| match self.tab {
                        Tab::Downloaded => true,
                        Tab::Unplayed => i.get_play_count() == 0,
                    })
                    .map(|i| i.clone())
                    .collect();
                html! {<ItemListCompact items={filtered_items} show_details={match self.tab {
                    Tab::Downloaded => true,
                    Tab::Unplayed => false
                }} on_selected={ctx.link().callback(|i| Message::SetSource(Some(i)))} />}
            }
            None => html!(),
        }
    }

    fn view_sliders(&self, ctx: &Context<Self>) -> Html {
        match self.show_sliders {
            true => {
                html! {
                    <div class="card-content">
                        {match (&self.source, self.duration) {
                            (Some(source), Some(duration)) => {
                                let current_time = match source.0.get_playback_time() {
                                    Some(curr_time) => curr_time,
                                    None => 0.0
                                };
                                html! {
                                <>
                                    <Range min="0" step="any" value={current_time.to_string()} max={duration.to_string()} onchange={ctx.link().callback(|e| Message::TimeChange(e))}/>
                                    <div class="columns is-mobile">
                                        <div class="column is-one-third has-text-left">{self.format_time(current_time)}</div>
                                        <div class="column is-one-third has-text-centered">{format!("{}@{}", self.format_time(duration / source.1.meta.playback_rate), source.1.meta.playback_rate)}</div>
                                        <div class="column is-one-third has-text-right">{self.format_time(duration)}</div>
                                    </div>
                                    <Range min="0" step="0.1" value={source.1.meta.volume.to_string()} max="1.0" onchange={ctx.link().callback(|e| Message::VolumeChange(e))}/>
                                    <div class="columns is-mobile">
                                        <div class="column is-one-third has-text-left"><Icon name="volume_down" style={IconStyle::Outlined}/></div>
                                        <div class="column is-one-third has-text-centered">{format!("{:.1}", source.1.meta.volume)}</div>
                                        <div class="column is-one-third has-text-right"><Icon name="volume_up" style={IconStyle::Outlined}/></div>
                                    </div>
                                    <Range min="0.5" step="0.1" value={source.1.meta.playback_rate.to_string()} max="2.5" onchange={ctx.link().callback(|e| Message::PlaybackRateChange(e))}/>
                                    <div class="columns is-mobile">
                                        <div class="column is-one-third has-text-left"><Icon name="play_arrow" style={IconStyle::Outlined}/></div>
                                        <div class="column is-one-third has-text-centered">{format!("{:.1}", source.1.meta.playback_rate)}</div>
                                        <div class="column is-one-third has-text-right"><Icon name="fast_forward" style={IconStyle::Outlined}/></div>
                                    </div>
                                </>
                            }
                        },
                            (_, _) => html! {
                                <>
                                    <input type="range" disabled={true} min="0" step="0.1" value="0.5" max="1.0" style="width: 100%"/>
                                    <div class="columns is-mobile">
                                        <div class="column is-one-third has-text-left">{""}</div>
                                        <div class="column is-one-third has-text-centered">{"?"}</div>
                                        <div class="column is-one-third has-text-right">{"?"}</div>
                                    </div>
                                    <input type="range" disabled={true} min="0" step="0.1" value="0.5" max="1.0" style="width: 100%"/>
                                    <div class="columns is-mobile">
                                        <div class="column is-one-third has-text-left"><Icon name="volume_down" style={IconStyle::Outlined}/></div>
                                        <div class="column is-one-third has-text-centered">{"?"}</div>
                                        <div class="column is-one-third has-text-right"><Icon name="volume_up" style={IconStyle::Outlined}/></div>
                                    </div>
                                    <input type="range" disabled={true} min="0.5" step="0.1" value="1.5" max="2.5" style="width: 100%"/>
                                    <div class="columns is-mobile">
                                        <div class="column is-one-third has-text-left"><Icon name="play_arrow" style={IconStyle::Outlined}/></div>
                                        <div class="column is-one-third has-text-centered">{"?"}</div>
                                        <div class="column is-one-third has-text-right"><Icon name="fast_forward" style={IconStyle::Outlined}/></div>
                                    </div>
                                </>
                            }
                        }}
                    </div>
                }
            }
            false => html! {},
        }
    }

    fn format_time(&self, time: f64) -> String {
        format!("{}:{:02}", (time / 60.0) as u64, (time % 60.0) as u64)
    }

    fn process_update(&mut self, ctx: &Context<Self>, msg: Message) -> Result<bool, JsError> {
        match msg {
            Message::SwitchTab(tab) => {
                self.tab = tab;
                Ok(true)
            }
            Message::ToggleShowSliders => {
                self.show_sliders = !self.show_sliders;
                Ok(true)
            }
            // Message::RepoMessage(response) => match response {
            //     repo::Response::Items(mut items) => {
            //         items.sort_by(|a, b| {
            //             a.get_date()
            //                 .partial_cmp(&b.get_date())
            //                 .unwrap_or(Ordering::Equal)
            //         });

            //         self.items = Some(items);
            //         self.set_item_from_playlist(ctx);

            //         Ok(true)
            //     }
            //     repo::Response::UpdatedItem(item) => {
            //         let mut res = false;

            //         if let Some(source) = &mut self.source {
            //             if source.0.get_id() == item.get_id() {
            //                 self.source = Some((item.clone(), source.1.clone()));
            //                 res = true;
            //             }
            //         }

            //         if let Some(self_items) = &mut self.items {
            //             let len_before = self_items.len();

            //             self_items.retain(|i| i.get_id() != item.get_id());
            //             res = len_before != self_items.len();

            //             match item.get_download_status() {
            //                 DownloadStatus::Ok => {
            //                     self_items.push(item.clone());
            //                     res = true;
            //                 }
            //                 _ => {}
            //             }
            //             self_items.sort_by(|a, b| a.get_date().cmp(&b.get_date()));
            //         }

            //         Ok(res)
            //     }
            //     repo::Response::UpdatedChannel(channel) => {
            //         let mut res = false;

            //         if let Some(source) = &mut self.source {
            //             if source.1.val.id == channel.val.id {
            //                 self.source = Some((source.0.clone(), channel));
            //                 res = true;
            //             }
            //         }

            //         Ok(res)
            //     }
            //     _ => Ok(false),
            // },
            Message::Pause => {
                // self.player.send(player::Request::Pause);
                Ok(false)
            }
            Message::Play => {
                // self.player.send(player::Request::Play);
                Ok(false)
            }
            Message::SetSource(source) => {
                if let Some(item) = source {
                    // self.player.send(player::Request::SetSource(item.clone()));
                }

                Ok(true)
            }
            // Message::PlayerMessage(player_message) => match player_message {
            //     player::Response::SourceSet(item, channel, duration) => {
            //         self.source = Some((item, channel));
            //         self.duration = Some(duration);
            //         Ok(true)
            //     }
            //     player::Response::Paused => {
            //         self.is_playing = false;
            //         Ok(true)
            //     }
            //     player::Response::Playing => {
            //         self.is_playing = true;
            //         Ok(true)
            //     }
            //     player::Response::End => {
            //         self.is_playing = false;
            //         match (&self.source, &self.items) {
            //             (Some(curr_item), Some(items)) => {
            //                 if let Some(new_item) = items.iter().find(|i| {
            //                     i.get_id() != curr_item.0.get_id() && i.get_play_count() == 0
            //                 }) {
            //                     self.player
            //                         .send(player::Request::SetSource(new_item.clone()));
            //                     self.player.send(player::Request::Play)
            //                 }
            //             }
            //             (_, _) => {}
            //         };
            //         Ok(false)
            //     }
            //     player::Response::Status(status) => {
            //         self.status_obtained = true;

            //         match status {
            //             Some(status) => {
            //                 self.source = Some((status.0, status.1));
            //                 self.duration = Some(status.2);
            //                 self.is_playing = status.3;
            //             }
            //             None => {
            //                 self.set_item_from_playlist(ctx);
            //             }
            //         }

            //         Ok(true)
            //     }
            // },
            Message::TimeChange(value) => {
                // self.player
                //     .send(player::Request::SetCurrentTime(value.parse()?));
                Ok(false)
            }
            Message::VolumeChange(value) => {
                // self.player.send(player::Request::SetVolume(value.parse()?));
                Ok(false)
            }
            Message::PlaybackRateChange(value) => {
                // self.player
                //     .send(player::Request::SetPlaybackRate(value.parse()?));
                Ok(false)
            }
        }
    }

    fn set_item_from_playlist(&mut self, ctx: &Context<Self>) {
        match (&self.items, self.status_obtained, &self.source) {
            (Some(items), true, None) => {
                let new_source = items
                    .iter()
                    .find(|i| i.get_play_count() == 0)
                    .map(|i| i.clone());
                ctx.link().send_message(Message::SetSource(new_source))
            }
            (_, _, _) => {}
        }
    }
}

impl Component for Player {
    type Message = Message;
    type Properties = ();

    fn view(&self, ctx: &Context<Self>) -> Html {
        let item_title = match &self.source {
            Some(source) => source.0.get_title(),
            _ => String::from("..."),
        };

        html! {
            <>
                <section class="section">
                    <div class="card">
                        <header class="card-header">
                            {match (&self.source, self.is_playing) {
                                (Some(_), true) => html! {
                                    <button class="card-header-icon" onclick={ctx.link().callback(|_| Message::Pause)}><Icon name="pause" style={IconStyle::Outlined}/></button>
                                },
                                (Some(_), false) => html! {
                                    <button class="card-header-icon" onclick={ctx.link().callback(|_| Message::Play)}><Icon name="play_arrow" style={IconStyle::Outlined}/></button>
                                },
                                (None, _) => html! {
                                    <button class="card-header-icon" disabled={true}><Icon name="play_arrow" style={IconStyle::Outlined}/></button>
                                }
                            }}
                            <p class="card-header-title">{item_title}</p>
                            <button class="card-header-icon" onclick={ctx.link().callback(|_| Message::ToggleShowSliders)}><Icon name={match self.show_sliders { true => "expand_less", false => "expand_more"}} style={IconStyle::Outlined}/></button>
                        </header>
                        { self.view_sliders(ctx) }
                    </div>
                </section>
                <section class="section">
                    {self.view_tabs(ctx)}
                    {self.view_item_list(ctx)}
                </section>
            </>
        }
    }

    fn create(ctx: &Context<Self>) -> Self {
        // let cb = ctx.link().callback(Message::RepoMessage);
        // let mut repo = repo::Repo::bridge(cb);

        // repo.send(repo::Request::GetItemsByDownloadOk);

        // let mut player = player::Player::bridge(ctx.link().callback(Message::PlayerMessage));

        // player.send(player::Request::GetStatus);

        Self {
            // _repo: repo,
            items: None,
            source: None,
            // player,
            duration: None,
            // notifier: notifier::Notifier::dispatcher(),
            is_playing: false,
            show_sliders: false,
            tab: Tab::Unplayed,
            status_obtained: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match self.process_update(ctx, msg) {
            Ok(res) => res,
            Err(e) => {
                // self.notifier.send(notifier::Request::NotifyError(e));
                false
            }
        }
    }
}
