use crate::{
    agents::{player, repo},
    objects::item::Item,
};
use uuid::Uuid;
use yew::{prelude::*, virtual_dom::VNode};

pub struct Player {
    _repo: Box<dyn Bridge<repo::Repo>>,
    player: Box<dyn Bridge<player::Player>>,
    link: ComponentLink<Self>,
    items: Option<Vec<Item>>,
    playing: Option<Item>,
    current_time: Option<f64>,
    duration: Option<f64>,
}
pub enum Message {
    NewMessage(repo::Response),
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
                        { items.iter().map(|i| {
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
                {match &self.playing {
                    Some(item) => html!(<>
                        <p class="title">{item.get_title()}</p>
                        {match (self.current_time, self.duration) {
                            (Some(current_time), Some(duration)) => html!(<div>
                                <button class="button" onclick={self.link.callback(move |_| Message::Pause)}>
                                    <span class="icon"><ion-icon size="large" name="pause"/></span>
                                </button>
                                {self.format_time(current_time)} {"/"} {self.format_time(duration)}
                                <progress class="progress" value=current_time.to_string() max=duration.to_string()>{"."}</progress>
                            </div>),
                            (_,_) => html!(<progress class="progress" max="100">{"."}</progress>)
                        }}
                    </>),
                    None => html!()
                }}
                </div>
            </div>
            </section>
            {self.view_item_list()}
            </>
        }
    }

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let cb = link.callback(Message::NewMessage);
        let mut repo = repo::Repo::bridge(cb);

        repo.send(repo::Request::GetItemsByDownloadOk);

        let player = player::Player::bridge(link.callback(Message::PlayerMessage));

        Self {
            link,
            _repo: repo,
            items: None,
            playing: None,
            player,
            current_time: None,
            duration: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::NewMessage(response) => match response {
                repo::Response::Items(items) => {
                    self.items = Some(items);
                    true
                }
                _ => false,
            },
            Message::Pause => {
                self.player.send(player::Request::Pause);
                false
            }
            Message::Play(id) => {
                self.playing = Some(
                    self.items
                        .as_ref()
                        .unwrap()
                        .iter()
                        .find(|i| i.get_id() == id)
                        .unwrap()
                        .clone(),
                );
                self.player.send(player::Request::Play {
                    id,
                    speed: 1.5,
                    volume: 1.0,
                    seconds: 0,
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
                    true
                }
            },
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }
}
