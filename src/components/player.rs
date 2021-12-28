use crate::{
    agents::{notifier, player, repo},
    components::icon::{Icon, IconStyle},
    objects::{Item, JsError},
};
use std::collections::HashMap;
use uuid::Uuid;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged, Dispatched, Dispatcher};

pub struct Player {
    repo: Box<dyn Bridge<repo::Repo>>,
    player: Box<dyn Bridge<player::Player>>,
    items: Option<HashMap<Uuid, Item>>,
    playing: Option<Uuid>,
    current_time: Option<f64>,
    volume: Option<f64>,
    playback_rate: Option<f64>,
    duration: Option<f64>,
    notifier: Dispatcher<notifier::Notifier>,
}
pub enum Message {
    RepoMessage(repo::Response),
    PlayerMessage(player::Response),
    Play(Uuid),
    Pause,
    TimeChange(Event),
    VolumeChange(Event),
    PlaybackRateChange(Event),
}

impl Player {
    fn view_item_list(&self, ctx: &Context<Self>) -> Html {
        match &self.items {
            Some(items) => html!(html! {
                <section class="section">
                    <div class="columns"><div class="column">
                        { items.iter().map(|(_, i)| {
                            let id = i.get_id();
                            html! { <div class="card">
                            <header class="card-header">
                                <p class="card-header-title">{&i.get_title()}</p>
                                <button class="card-header-icon" aria-label="play" onclick={ctx.link().callback(move |_| Message::Play(id))}>
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

    fn process_update(&mut self, _ctx: &Context<Self>, msg: Message) -> Result<bool, JsError> {
        match msg {
            Message::RepoMessage(response) => match response {
                repo::Response::Items(items) => {
                    self.items = Some(items.iter().map(|i| (i.get_id(), i.clone())).collect());
                    Ok(true)
                }
                repo::Response::Item(_item) => {
                    // self.items.as_mut().unwrap().insert(item.get_id(), item);
                    Ok(false)
                }
                _ => Ok(false),
            },
            Message::Pause => {
                self.player.send(player::Request::Pause);
                Ok(false)
            }
            Message::Play(id) => {
                let item = &self
                    .items
                    .as_ref()
                    .ok_or("error getting item list reference")?[&id];
                let current_time = match item.get_current_time() {
                    Some(ct) => ct,
                    None => 0.0,
                };

                self.current_time = Some(current_time);
                self.playing = Some(item.get_id());
                self.volume = Some(1.0);
                self.playback_rate = Some(1.5);
                self.player.send(player::Request::SetSource(item.get_id()));
                Ok(false)
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
                    Ok(false)
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

                    let item = &mut self
                        .items
                        .as_mut()
                        .ok_or("could not get mutable reference")?
                        .get_mut(&id)
                        .ok_or("could not get mutable reference to item")?;

                    item.set_current_time(Some(current_time));

                    self.repo.send(repo::Request::UpdateItem((**item).clone()));

                    Ok(true)
                }
            },
            Message::TimeChange(ev) => {
                if let Ok(i) = get_input_element_from_event(ev) {
                    let current_time = i.value().parse()?;

                    self.player
                        .send(player::Request::SetCurrentTime(current_time));
                }
                Ok(false)
            }
            Message::VolumeChange(ev) => {
                if let Ok(i) = get_input_element_from_event(ev) {
                    let volume = i.value().parse()?;

                    self.player.send(player::Request::SetVolume(volume));
                }
                Ok(false)
            }
            Message::PlaybackRateChange(ev) => {
                if let Ok(i) = get_input_element_from_event(ev) {
                    let playback_rate = i.value().parse()?;

                    self.player
                        .send(player::Request::SetPlaybackRate(playback_rate));
                }
                Ok(false)
            }
        }
    }
}

impl Component for Player {
    type Message = Message;
    type Properties = ();

    fn view(&self, ctx: &Context<Self>) -> Html {
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
                                        <button class="button" onclick={ctx.link().callback(move |_| Message::Pause)}>
                                            <Icon name="pause" style={IconStyle::Outlined}/>
                                        </button>
                                    </div>
                                    <div class="tile is-child is-1" style="text-align: center">
                                        {self.format_time(current_time)}
                                    </div>
                                    <div class="tile is-child">
                                        <input type="range" min="0" value={current_time.to_string()} max={duration.to_string()} style="width: 100%" onchange={ctx.link().callback(|e| Message::TimeChange(e))}/>
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
                                        <input type="range" min="0" step="0.05" value={volume.to_string()} max="1.0" style="width: 100%" onchange={ctx.link().callback(|e| Message::VolumeChange(e))}/>
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
                                        <input type="range" min="0.5" step="0.05" value={playback_rate.to_string()} max="2.5" style="width: 100%" onchange={ctx.link().callback(|e| Message::PlaybackRateChange(e))}/>
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
            {self.view_item_list(ctx)}
            </>
        }
    }

    fn create(ctx: &Context<Self>) -> Self {
        let cb = ctx.link().callback(Message::RepoMessage);
        let mut repo = repo::Repo::bridge(cb);

        repo.send(repo::Request::GetItemsByDownloadOk);

        let player = player::Player::bridge(ctx.link().callback(Message::PlayerMessage));

        Self {
            repo,
            items: None,
            playing: None,
            player,
            current_time: None,
            playback_rate: None,
            volume: None,
            duration: None,
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
}

fn get_input_element_from_event(ev: Event) -> Result<HtmlInputElement, JsError> {
    let target = ev.target().ok_or("could not get target object")?;

    target
        .dyn_into::<HtmlInputElement>()
        .map_err(|_| JsError::from("error casting target to input element"))
}
