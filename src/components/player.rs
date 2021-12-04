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
                <ion-list>
                        { items.iter().map(|(_, i)| {
                            let id = i.get_id();
                            html! { <ion-item>
                            <ion-label>{&i.get_title()}</ion-label>
                                <ion-button onclick={self.link.callback(move |_| Message::Play(id))}>
                                    <ion-icon name="play"/>
                                </ion-button>
                        </ion-item>}}).collect::<Html>() }
                </ion-list>
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
        html! {<>
                <ion-card>
                    {match (&self.playing, &self.items) {
                        (Some(item), Some(items)) => {
                            let item = &items[item];
                            html!(<><ion-card-header>
                                <ion-item>
                                    <ion-button slot="start" onclick={self.link.callback(move |_| Message::Pause)}><ion-icon name="pause"></ion-icon></ion-button>
                                    <ion-card-title>{item.get_title()}</ion-card-title>
                                </ion-item>
                                </ion-card-header><ion-card-content>
                            {match (self.current_time, self.duration) {
                                (Some(current_time), Some(duration)) => html!(<>
                                    <ion-item>
                                        <ion-range min="0" max={duration.to_string()} value={current_time.to_string()} step="1">
                                            <ion-label slot="start">{self.format_time(current_time)}</ion-label>
                                            <ion-label slot="end">{self.format_time(duration)}</ion-label>
                                        </ion-range>
                                    </ion-item>
                                    <ion-item>
                                        <ion-range min="0.5" max="2.5" value="1.0" step="1">
                                            <ion-icon name="volume-low" slot="start"></ion-icon>
                                            <ion-icon name="volume-high" slot="end"></ion-icon>
                                        </ion-range>
                                    </ion-item>
                                    <ion-item>
                                        <ion-range min="0.5" max="2.5" value="1.5" step="1">
                                            <ion-icon name="play" slot="start"></ion-icon>
                                            <ion-icon name="play-forward" slot="end"></ion-icon>
                                    </ion-range>
                                    </ion-item>
                                </>),
                                (_, _) => html!()
                            }}
                            </ion-card-content></>)
                        },
                        (_, _) => html!()
                    }}
                </ion-card>
                {self.view_item_list()}
        </>
                    //                             {self.format_time(current_time)}
                    //                         </div>
                    //                         <div class="tile is-child">
                    //                             <progress class="progress" value=current_time.to_string() max=duration.to_string()>{"."}</progress>
                    //                         </div>
                    //                         <div class="tile is-child is-1" style="text-align: center">
                    //                             {self.format_time(duration)}
                    //                         </div>
                    //                     </div>
                    //                     <div class="tile is-parent">
                    //                         <div class="tile is-child">
                    //                             <button class="button" onclick={self.link.callback(move |_| Message::Pause)}>
                    //                                 <span class="icon"><ion-icon size="large" name="play-skip-back"/></span>
                    //                             </button>
                    //                             <button class="button" onclick={self.link.callback(move |_| Message::Pause)}>
                    //                                 <span class="icon"><ion-icon size="large" name="play-back"/></span>
                    //                             </button>
                    //                             <button class="button" onclick={self.link.callback(move |_| Message::Pause)}>
                    //                                 <span class="icon"><ion-icon size="large" name="pause"/></span>
                    //                             </button>
                    //                             <button class="button" onclick={self.link.callback(move |_| Message::Pause)}>
                    //                                 <span class="icon"><ion-icon size="large" name="play-forward"/></span>
                    //                             </button>
                    //                             <button class="button" onclick={self.link.callback(move |_| Message::Pause)}>
                    //                                 <span class="icon"><ion-icon size="large" name="play-skip-forward"/></span>
                    //                             </button>
                    //                         </div>
                    //                     </div>
                    //                 </div></div>),
                    //                 (_,_) => html!()
                    //             }}
                    //         </>)},
                    //         (_, _) => html!()
                    //     }}
                    //     </div>
                    // </div>
                    // </section>
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
