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

pub enum Tab {
    Unplayed,
    Downloaded,
}

pub struct Player {
    _repo: Box<dyn Bridge<repo::Repo>>,
    player: Box<dyn Bridge<player::Player>>,
    items: Option<HashMap<Uuid, Item>>,
    source: Option<Item>,
    volume: Option<f64>,
    playback_rate: Option<f64>,
    duration: Option<f64>,
    notifier: Dispatcher<notifier::Notifier>,
    is_playing: bool,
    show_sliders: bool,
    allow_update: bool,
    tab: Tab,
}
pub enum Message {
    RepoMessage(repo::Response),
    PlayerMessage(player::Response),
    SetSource(Uuid),
    Play,
    Pause,
    OnFocus(FocusEvent),
    TimeChange(Event),
    VolumeChange(Event),
    PlaybackRateChange(Event),
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
            Some(items) => html! {
                <div class="columns"><div class="column">
                    { items.iter().filter(|(_, i)| match self.tab {
                        Tab::Downloaded => true,
                        Tab::Unplayed => i.get_play_count() == 0
                    }).map(|(_, i)| {
                        let id = i.get_id();
                        html! { <div class="card" onclick={ctx.link().callback(move |_| Message::SetSource(id))}>
                        <header class="card-header">
                            <p class="card-header-title">{&i.get_title()}</p>
                        </header>
                        <div class="card-content">
                            <div class="tags has-addons">
                                <span class="tag">{format!("{}x", &i.get_play_count())}</span>
                                <span class="tag is-primary">{"played"}</span>
                            </div>
                        </div>
                    </div> }}).collect::<Html>() }
                </div></div>
            },
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
                                let current_time = match source.get_current_time() {
                                    Some(curr_time) => curr_time,
                                    None => 0.0
                                };
                                html! {
                                <>
                                    <input type="range" min="0" step="any" value={current_time.to_string()} max={duration.to_string()} style="width: 100%" onfocus={ctx.link().callback(|e| Message::OnFocus(e))} onchange={ctx.link().callback(|e| Message::TimeChange(e))}/>
                                    <div class="columns is-mobile">
                                        <div class="column is-one-third has-text-left">{""}</div>
                                        <div class="column is-one-third has-text-centered">{self.format_time(current_time)}</div>
                                        <div class="column is-one-third has-text-right">{self.format_time(duration)}</div>
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
            Message::RepoMessage(response) => match response {
                repo::Response::Items(items) => {
                    let mut map = HashMap::new();
                    let mut selected = None;

                    for item in items {
                        if item.get_play_count() == 0 && selected.is_none() {
                            selected = Some(item.get_id());
                        }
                        map.insert(item.get_id(), item);
                    }

                    self.items = Some(map);

                    if let Some(item_id) = selected {
                        ctx.link().send_message(Message::SetSource(item_id))
                    }

                    Ok(true)
                }
                repo::Response::UpdateItem(_item) => {
                    // self.items.as_mut().unwrap().insert(item.get_id(), item);
                    Ok(false)
                }
                repo::Response::ModifiedItems(items) => {
                    let mut res = false;

                    if let Some(source) = &mut self.source {
                        if let Some(updated_item) =
                            items.iter().find(|i| i.get_id() == source.get_id())
                        {
                            self.source = Some((*updated_item).clone());
                            res = true;
                        }
                    }

                    if let Some(self_items) = &mut self.items {
                        for item in items {
                            if self_items.contains_key(&item.get_id()) {
                                self_items.remove(&item.get_id());
                                self_items.insert(item.get_id(), item);
                            }
                        }
                    }

                    Ok(res)
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
                let volume = 1.0;
                let playback_rate = 1.5;

                self.source = Some((*item).clone());
                self.volume = Some(volume);
                self.playback_rate = Some(playback_rate);
                self.player
                    .send(player::Request::SetSource((*item).clone()));
                Ok(true)
            }
            Message::PlayerMessage(player_message) => match player_message {
                player::Response::SourceSet(item, duration) => {
                    self.source = Some(item);
                    self.duration = Some(duration);
                    Ok(true)
                }
                player::Response::Paused => {
                    self.is_playing = false;
                    Ok(true)
                }
                player::Response::Playing => {
                    self.is_playing = true;
                    Ok(true)
                }
                player::Response::End => {
                    self.is_playing = false;
                    match (&self.source, &self.items) {
                        (Some(curr_item), Some(items)) => {
                            if let Some((_, new_item)) = items.iter().find(|(_, i)| {
                                i.get_id() != curr_item.get_id() && i.get_play_count() == 0
                            }) {
                                self.player
                                    .send(player::Request::SetSource(new_item.clone()));
                                self.player.send(player::Request::Play)
                            }
                        }
                        (_, _) => {}
                    };
                    Ok(false)
                }
            },
            Message::OnFocus(_ev) => {
                self.allow_update = false;
                Ok(false)
            }
            Message::TimeChange(ev) => {
                if let Ok(i) = get_input_element_from_event(ev) {
                    let current_time = i.value().parse()?;

                    self.allow_update = true;
                    i.blur()?;
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
        let item_title = match &self.source {
            Some(source) => source.get_title(),
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
        let cb = ctx.link().callback(Message::RepoMessage);
        let mut repo = repo::Repo::bridge(cb);

        repo.send(repo::Request::GetItemsByDownloadOk);

        let player = player::Player::bridge(ctx.link().callback(Message::PlayerMessage));

        Self {
            _repo: repo,
            items: None,
            source: None,
            player,
            playback_rate: None,
            volume: None,
            duration: None,
            notifier: notifier::Notifier::dispatcher(),
            is_playing: false,
            show_sliders: false,
            allow_update: true,
            tab: Tab::Unplayed,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match self.process_update(ctx, msg) {
            Ok(res) => res && self.allow_update,
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
