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
    duration: Option<f64>,
}
pub enum Message {
    RepoMessage(repo::Response),
    PlayerMessage(player::Response),
    Play(Uuid),
    Pause,
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
                                    <span class="icon"><ion-icon size="large" name="play"/></span>
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
                        {match (self.current_time, self.duration) {
                            (Some(current_time), Some(duration)) => html!(<div class="tile is-ancestor">
                            <div class="tile is-vertical">
                                <div class="tile is-parent">
                                    <div class="tile is-child is-1" style="text-align: center">
                                        {self.format_time(current_time)}
                                    </div>
                                    <div class="tile is-child">
                                        <progress class="progress" value=current_time.to_string() max=duration.to_string()>{"."}</progress>
                                    </div>
                                    <div class="tile is-child is-1" style="text-align: center">
                                        {self.format_time(duration)}
                                    </div>
                                </div>
                                <div class="tile is-parent">
                                    <div class="tile is-child">
                                        <button class="button" onclick={self.link.callback(move |_| Message::Pause)}>
                                            <span class="icon"><ion-icon size="large" name="play-skip-back"/></span>
                                        </button>
                                        <button class="button" onclick={self.link.callback(move |_| Message::Pause)}>
                                            <span class="icon"><ion-icon size="large" name="play-back"/></span>
                                        </button>
                                        <button class="button" onclick={self.link.callback(move |_| Message::Pause)}>
                                            <span class="icon"><ion-icon size="large" name="pause"/></span>
                                        </button>
                                        <button class="button" onclick={self.link.callback(move |_| Message::Pause)}>
                                            <span class="icon"><ion-icon size="large" name="play-forward"/></span>
                                        </button>
                                        <button class="button" onclick={self.link.callback(move |_| Message::Pause)}>
                                            <span class="icon"><ion-icon size="large" name="play-skip-forward"/></span>
                                        </button>
                                    </div>
                                </div>
                            </div></div>),
                            (_,_) => html!(<progress class="progress" max="100">{"."}</progress>)
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
                repo::Response::Item(item) => {
                    self.items.as_mut().unwrap().insert(item.get_id(), item);
                    true
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

                self.playing = Some(item.get_id());
                self.player.send(player::Request::Play {
                    id,
                    playback_rate: 1.5,
                    volume: 1.0,
                    current_time,
                });
                false
            }
            Message::PlayerMessage(player_message) => match player_message {
                player::Response::Playing {
                    current_time,
                    id,
                    duration,
                } => {
                    self.duration = Some(duration);
                    self.current_time = Some(current_time);
                    self.playing = Some(id);

                    let mut item = self.items.as_ref().unwrap()[&id].clone();

                    item.set_current_time(Some(current_time));
                    self.repo.send(repo::Request::UpdateItem(item.clone()));

                    true
                }
            },
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }
}
