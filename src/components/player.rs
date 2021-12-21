use crate::components::icon::{Icon, IconStyle};
use std::collections::HashMap;

use crate::{
    agents::{player, repo},
    objects::item::Item,
};
use uuid::Uuid;
use yew::{prelude::*, virtual_dom::VNode};

pub struct Player {
    repo: Box<dyn Bridge<repo::Repo>>,
    player: Box<dyn Bridge<player::Player>>,
    link: ComponentLink<Self>,
    items: Option<HashMap<Uuid, Item>>,
    playing: Option<Uuid>,
    current_time: Option<f64>,
    volume: Option<f64>,
    playback_rate: Option<f64>,
    duration: Option<f64>,
}
pub enum Message {
    RepoMessage(repo::Response),
    PlayerMessage(player::Response),
    Play(Uuid),
    Pause,
    TimeChange(ChangeData),
    VolumeChange(ChangeData),
    PlaybackRateChange(ChangeData),
}

impl Player {
    fn view_item_list(&self) -> Html {
        match &self.items {
            Some(items) => html!(html! {
                <section class="section">
                    <div class="columns"><div class="column">
                        { items.iter().map(|(_, i)| {
                            let id = i.get_id();
                            html! { <div class="card">
                            <header class="card-header">
                                <p class="card-header-title">{&i.get_title()}</p>
                                <button class="card-header-icon" aria-label="play" onclick={self.link.callback(move |_| Message::Play(id))}>
                                    <Icon name="play_arrow" style={IconStyle::Outlined}/>
                                </button>
                            </header>
                        </div> }}).collect::<Html>() }
                    </div></div>
                </section>
            }),
            None => html!(),
        }
    }

    fn format_time(&self, time: f64) -> String {
        format!("{}:{:02}", (time / 60.0) as u64, (time % 60.0) as u64)
    }
}

impl Component for Player {
    type Message = Message;
    type Properties = ();

    fn view(&self) -> VNode {
        html! {
            <>
            <section class="section">
            <div class="card">
                <header class="card-header">
                </header>
                <div class="card-content">
                {match (&self.playing, &self.items) {
                    (Some(item), Some(items)) => {
                        let item = &items[item];
                        html!(<>
                        <p class="title">{item.get_title()}</p>
                        {match (self.current_time, self.duration, self.volume, self.playback_rate) {
                            (Some(current_time), Some(duration), Some(volume), Some(playback_rate)) => html!(<div class="tile is-ancestor">
                            <div class="tile is-vertical">
                                <div class="tile is-parent">
                                    <div class="tile is-child is-1">
                                        <button class="button" onclick={self.link.callback(move |_| Message::Pause)}>
                                            <Icon name="pause" style={IconStyle::Outlined}/>
                                        </button>
                                    </div>
                                    <div class="tile is-child is-1" style="text-align: center">
                                        {self.format_time(current_time)}
                                    </div>
                                    <div class="tile is-child">
                                        <input type="range" min="0" value=current_time.to_string() max=duration.to_string() style="width: 100%" onchange={self.link.callback(|e| Message::TimeChange(e))}/>
                                    </div>
                                    <div class="tile is-child is-1" style="text-align: center">
                                        {self.format_time(duration)}
                                    </div>
                                </div>
                                <div class="tile is-parent">
                                    <div class="tile is-child is-1" style="text-align: center">
                                        <Icon name="volume_down" style={IconStyle::Outlined}/>
                                    </div>
                                    <div class="tile is-child">
                                        <input type="range" min="0" step="0.05" value=volume.to_string() max="1.0" style="width: 100%" onchange={self.link.callback(|e| Message::VolumeChange(e))}/>
                                    </div>
                                    <div class="tile is-child is-1" style="text-align: center">
                                        <Icon name="volume_up" style={IconStyle::Outlined}/>
                                    </div>
                                </div>
                                <div class="tile is-parent">
                                    <div class="tile is-child is-1" style="text-align: center">
                                        <Icon name="play_arrow" style={IconStyle::Outlined}/>
                                    </div>
                                    <div class="tile is-child">
                                        <input type="range" min="0.5" step="0.05" value=playback_rate.to_string() max="2.5" style="width: 100%" onchange={self.link.callback(|e| Message::PlaybackRateChange(e))}/>
                                    </div>
                                    <div class="tile is-child is-1" style="text-align: center">
                                        <Icon name="fast_forward" style={IconStyle::Outlined}/>
                                    </div>
                                </div>
                            </div></div>),
                            (_,_,_,_) => html!(<progress class="progress" max="100">{"."}</progress>)
                        }}
                    </>)},
                    (_, _) => html!()
                }}
                </div>
            </div>
            </section>
            {self.view_item_list()}
            </>
        }
    }

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let cb = link.callback(Message::RepoMessage);
        let mut repo = repo::Repo::bridge(cb);

        repo.send(repo::Request::GetItemsByDownloadOk);

        let player = player::Player::bridge(link.callback(Message::PlayerMessage));

        Self {
            link,
            repo,
            items: None,
            playing: None,
            player,
            current_time: None,
            playback_rate: None,
            volume: None,
            duration: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::RepoMessage(response) => match response {
                repo::Response::Items(items) => {
                    self.items = Some(items.iter().map(|i| (i.get_id(), i.clone())).collect());
                    true
                }
                repo::Response::Item(_item) => {
                    // self.items.as_mut().unwrap().insert(item.get_id(), item);
                    false
                }
                _ => false,
            },
            Message::Pause => {
                self.player.send(player::Request::Pause);
                false
            }
            Message::Play(id) => {
                let item = &self.items.as_ref().unwrap()[&id];
                let current_time = match item.get_current_time() {
                    Some(ct) => ct,
                    None => 0.0,
                };

                self.current_time = Some(current_time);
                self.playing = Some(item.get_id());
                self.volume = Some(1.0);
                self.playback_rate = Some(1.5);
                self.player.send(player::Request::SetSource(item.get_id()));
                false
            }
            Message::PlayerMessage(player_message) => match player_message {
                player::Response::SourceSet => {
                    match (&self.volume, &self.playback_rate, &self.current_time) {
                        (Some(volume), Some(playback_rate), Some(current_time)) => {
                            self.player.send(player::Request::Play {
                                playback_rate: playback_rate.clone(),
                                volume: volume.clone(),
                                current_time: current_time.clone(),
                            });
                        }
                        (_, _, _) => {}
                    }
                    false
                }
                player::Response::Update {
                    current_time,
                    id,
                    duration,
                    playback_rate,
                    volume,
                } => {
                    self.duration = Some(duration);
                    self.current_time = Some(current_time);
                    self.playing = Some(id);
                    self.playback_rate = Some(playback_rate);
                    self.volume = Some(volume);

                    let item = &mut self.items.as_mut().unwrap().get_mut(&id).unwrap();

                    item.set_current_time(Some(current_time));

                    self.repo.send(repo::Request::UpdateItem((**item).clone()));

                    true
                }
            },
            Message::TimeChange(cd) => match cd {
                ChangeData::Value(value) => {
                    let current_time = value.parse().unwrap();

                    self.player
                        .send(player::Request::SetCurrentTime(current_time));
                    false
                }
                _ => false,
            },
            Message::VolumeChange(cd) => match cd {
                ChangeData::Value(value) => {
                    let volume = value.parse().unwrap();
                    self.player.send(player::Request::SetVolume(volume));
                    false
                }
                _ => false,
            },
            Message::PlaybackRateChange(cd) => match cd {
                ChangeData::Value(value) => {
                    let playback_rate = value.parse().unwrap();
                    self.player
                        .send(player::Request::SetPlaybackRate(playback_rate));
                    false
                }
                _ => false,
            },
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }
}
