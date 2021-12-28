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
    source_id: Option<Uuid>,
    current_time: Option<f64>,
    volume: Option<f64>,
    playback_rate: Option<f64>,
    duration: Option<f64>,
    notifier: Dispatcher<notifier::Notifier>,
    is_playing: bool,
    show_sliders: bool,
}
pub enum Message {
    RepoMessage(repo::Response),
    PlayerMessage(player::Response),
    SetSource(Uuid),
    Play,
    Pause,
    TimeChange(Event),
    VolumeChange(Event),
    PlaybackRateChange(Event),
    ToggleShowSliders,
}

impl Player {
    fn view_item_list(&self, ctx: &Context<Self>) -> Html {
        match &self.items {
            Some(items) => html!(html! {
                <section class="section">
                    <div class="columns"><div class="column">
                        { items.iter().map(|(_, i)| {
                            let id = i.get_id();
                            html! { <div class="card" onclick={ctx.link().callback(move |_| Message::SetSource(id))}>
                            <header class="card-header">
                                <p class="card-header-title">{&i.get_title()}</p>
                            </header>
                        </div> }}).collect::<Html>() }
                    </div></div>
                </section>
            }),
            None => html!(),
        }
    }

    fn view_sliders(&self, ctx: &Context<Self>) -> Html {
        match self.show_sliders {
            true => {
                html! {
                    <div class="card-content">
                        {match (self.current_time, self.duration) {
                            (Some(current_time), Some(duration)) => html! {
                                <>
                                    <input type="range" min="0" step="0.1" value={current_time.to_string()} max={duration.to_string()} style="width: 100%" onchange={ctx.link().callback(|e| Message::TimeChange(e))}/>
                                    <div class="columns is-mobile">
                                        <div class="column is-one-third has-text-left">{""}</div>
                                        <div class="column is-one-third has-text-centered">{self.format_time(current_time)}</div>
                                        <div class="column is-one-third has-text-right">{self.format_time(duration)}</div>
                                    </div>
                                </>
                            },
                            (_, _) => html! {
                                <>
                                    <input type="range" disabled={true} min="0" step="0.1" value="0.5" max="1.0" style="width: 100%"/>
                                    <div class="columns is-mobile">
                                        <div class="column is-one-third has-text-left">{""}</div>
                                        <div class="column is-one-third has-text-centered">{"?"}</div>
                                        <div class="column is-one-third has-text-right">{"?"}</div>
                                    </div>
                                </>
                            }
                        }}
                        {match self.volume {
                            Some(volume) => html! {
                                <>
                                    <input type="range" min="0" step="0.1" value={volume.to_string()} max="1.0" style="width: 100%" onchange={ctx.link().callback(|e| Message::VolumeChange(e))}/>
                                    <div class="columns is-mobile">
                                        <div class="column is-one-third has-text-left"><Icon name="volume_down" style={IconStyle::Outlined}/></div>
                                        <div class="column is-one-third has-text-centered">{format!("{:.1}", volume)}</div>
                                        <div class="column is-one-third has-text-right"><Icon name="volume_up" style={IconStyle::Outlined}/></div>
                                    </div>
                                </>
                            },
                            _ => html! {
                                <>
                                    <input type="range" disabled={true} min="0" step="0.1" value="0.5" max="1.0" style="width: 100%"/>
                                    <div class="columns is-mobile">
                                        <div class="column is-one-third has-text-left"><Icon name="volume_down" style={IconStyle::Outlined}/></div>
                                        <div class="column is-one-third has-text-centered">{"?"}</div>
                                        <div class="column is-one-third has-text-right"><Icon name="volume_up" style={IconStyle::Outlined}/></div>
                                    </div>
                                </>
                            }
                        }}
                        {match self.playback_rate {
                            Some(playback_rate) => html! {
                                <>
                                    <input type="range" min="0.5" step="0.1" value={playback_rate.to_string()} max="2.5" style="width: 100%" onchange={ctx.link().callback(|e| Message::PlaybackRateChange(e))}/>
                                    <div class="columns is-mobile">
                                        <div class="column is-one-third has-text-left"><Icon name="play_arrow" style={IconStyle::Outlined}/></div>
                                        <div class="column is-one-third has-text-centered">{format!("{:.1}", playback_rate)}</div>
                                        <div class="column is-one-third has-text-right"><Icon name="fast_forward" style={IconStyle::Outlined}/></div>
                                    </div>
                                </>
                            },
                            _ => html! {
                                <>
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

    fn process_update(&mut self, _ctx: &Context<Self>, msg: Message) -> Result<bool, JsError> {
        match msg {
            Message::ToggleShowSliders => {
                self.show_sliders = !self.show_sliders;
                Ok(true)
            }
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
            Message::Play => {
                self.player.send(player::Request::Play);
                Ok(false)
            }
            Message::SetSource(id) => {
                let item = &self
                    .items
                    .as_ref()
                    .ok_or("error getting item list reference")?[&id];
                let current_time = match item.get_current_time() {
                    Some(ct) => ct,
                    None => 0.0,
                };
                let volume = 1.0;
                let playback_rate = 1.5;

                self.current_time = Some(current_time);
                self.source_id = Some(item.get_id());
                self.volume = Some(volume);
                self.playback_rate = Some(playback_rate);
                self.player.send(player::Request::SetSource {
                    id: item.get_id(),
                    current_time,
                    playback_rate,
                    volume,
                });
                Ok(false)
            }
            Message::PlayerMessage(player_message) => match player_message {
                player::Response::Update {
                    current_time,
                    id,
                    duration,
                    playback_rate,
                    volume,
                    is_playing,
                } => {
                    self.duration = Some(duration);
                    self.current_time = Some(current_time);
                    self.source_id = Some(id);
                    self.playback_rate = Some(playback_rate);
                    self.volume = Some(volume);
                    self.is_playing = is_playing;

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
        let item_title = match (&self.source_id, &self.items) {
            (Some(source_id), Some(items)) => items[source_id].get_title(),
            (_, _) => String::from("..."),
        };

        html! {
            <>
                <section class="section">
                    <div class="card">
                        <header class="card-header">
                            {match (self.source_id, self.is_playing) {
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
            source_id: None,
            player,
            current_time: None,
            playback_rate: None,
            volume: None,
            duration: None,
            notifier: notifier::Notifier::dispatcher(),
            is_playing: false,
            show_sliders: false,
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
