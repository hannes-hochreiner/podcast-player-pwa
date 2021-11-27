use crate::{agents::repo, objects::item::Item};
use uuid::Uuid;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{MediaSource, Url};
use yew::{prelude::*, virtual_dom::VNode};

pub struct Player {
    repo: Box<dyn Bridge<repo::Repo>>,
    link: ComponentLink<Self>,
    update_closure: Closure<dyn Fn(web_sys::Event)>,
    mediasource_opened_closure: Closure<dyn Fn(web_sys::Event)>,
    media_source: MediaSource,
    audio_ref: NodeRef,
    items: Option<Vec<Item>>,
    played_item_id: Option<Uuid>,
}
pub enum Message {
    NewMessage(repo::Response),
    Play,
    Info,
    Update(Event),
    SetSource(Uuid),
    SourceOpened(Event),
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
                            <div class="card-content">
                                <p class="title">{&i.get_title()}</p>
                                <p class="buttons">
                                    <button class="button is-primary" onclick={self.link.callback(move |_| Message::SetSource(id))}><span class="icon"><ion-icon size="large" name="star"/></span><span>{"play"}</span></button>
                                </p>
                            </div>
                        </div> }}).collect::<Html>() }
                    </div></div>
                </section>
            }),
            None => html!(),
        }
    }
}

impl Component for Player {
    type Message = Message;
    type Properties = ();

    fn view(&self) -> VNode {
        html! {
            <>
                <audio controls={true} ref=self.audio_ref.clone()/>
                <button class="button" onclick={self.link.callback(move |_| Message::Play)}>{"set source"}</button>
                <button class="button" onclick={self.link.callback(move |_| Message::Info)}>{"info"}</button>
                {self.view_item_list()}
            </>
        }
    }

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let cb = link.callback(Message::NewMessage);
        let ms = MediaSource::new().unwrap();
        let callback_update = link.callback(move |e| Message::Update(e));
        let update_closure =
            Closure::wrap(
                Box::new(move |event: web_sys::Event| callback_update.emit(event))
                    as Box<dyn Fn(_)>,
            );
        let callback_mediasource_opened = link.callback(move |e| Message::SourceOpened(e));
        let mediasource_opened_closure = Closure::wrap(Box::new(move |event: web_sys::Event| {
            callback_mediasource_opened.emit(event)
        }) as Box<dyn Fn(_)>);

        let mut repo = repo::Repo::bridge(cb);

        repo.send(repo::Request::GetItemsByDownloadOk);

        Self {
            link,
            repo,
            media_source: ms,
            audio_ref: NodeRef::default(),
            update_closure,
            mediasource_opened_closure,
            items: None,
            played_item_id: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::NewMessage(response) => match response {
                repo::Response::Enclosure(data) => {
                    let sb = self.media_source.add_source_buffer("audio/mpeg").unwrap();
                    let ae = self.audio_ref.cast::<web_sys::HtmlAudioElement>().unwrap();
                    ae.set_playback_rate(1.5);
                    ae.set_preload("metadata");
                    sb.append_buffer_with_array_buffer(&data).unwrap();
                    sb.set_onupdate(Some(self.update_closure.as_ref().unchecked_ref()));
                    true
                }
                repo::Response::Items(items) => {
                    self.items = Some(items);
                    true
                }
                _ => false,
            },
            Message::SetSource(id) => {
                let ae = self.audio_ref.cast::<web_sys::HtmlAudioElement>().unwrap();
                ae.pause().unwrap();
                self.played_item_id = Some(id);
                self.media_source = MediaSource::new().unwrap();
                ae.set_src(&Url::create_object_url_with_source(&self.media_source).unwrap());
                self.media_source.set_onsourceopen(Some(
                    self.mediasource_opened_closure.as_ref().unchecked_ref(),
                ));
                false
            }
            Message::SourceOpened(_ev) => match self.played_item_id {
                Some(id) => {
                    self.repo.send(repo::Request::GetEnclosure(id));
                    false
                }
                None => false,
            },
            Message::Play => {
                // self._repo.send(Request::GetEnclosure(
                //     uuid::Uuid::parse_str("200541bb-662b-40b9-a2d8-7d38444216f6").unwrap(),
                // ));
                false
            }
            Message::Info => {
                let ae = self.audio_ref.cast::<web_sys::HtmlAudioElement>().unwrap();

                log::info!(
                    "preload: {} ms: {}, time: {}, ae state: {}, ms state: {:?}",
                    ae.preload(),
                    self.media_source.duration(),
                    ae.current_time(),
                    ae.ready_state(),
                    self.media_source.ready_state()
                );
                false
            }
            Message::Update(_e) => {
                self.media_source.end_of_stream().unwrap();
                false
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }
}
