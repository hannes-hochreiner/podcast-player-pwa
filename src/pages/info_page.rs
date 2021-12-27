use crate::{agents::notifier, components::NavBar, components::Notification, objects::JsError};
use serde::Deserialize;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::ConnectionType;
use yew::prelude::*;
use yew_agent::{Dispatched, Dispatcher};

/// TODO: show persist information
/// TODO: move persist request to repository
pub struct InfoPage {
    estimate: Option<Estimate>,
    connection_type: Option<ConnectionType>,
    notifier: Dispatcher<notifier::Notifier>,
}
pub enum Message {
    GetEstimate(Result<JsValue, JsValue>),
    GetPersisted(Result<JsValue, JsValue>),
    GetPersist(Result<JsValue, JsValue>),
}
#[derive(Properties, Clone, PartialEq)]
pub struct Props {}

#[derive(Debug, Deserialize, Clone)]
struct Estimate {
    quota: u32,
    usage: u32,
}

impl InfoPage {
    fn view_network_info(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <section class="section">
                <div class="title">{"Connection Information"}</div>
                <p>{"Connection Type"}</p>
                <p>{match &self.connection_type {
                    Some(connection_type) => match connection_type {
                        ConnectionType::Wifi=>"Wifi",
                        ConnectionType::Cellular => "Cellular",
                        ConnectionType::Bluetooth => "Bluetooth",
                        ConnectionType::Ethernet => "Ethernet",
                        ConnectionType::Other => "Other",
                        ConnectionType::None => "None",
                        ConnectionType::Unknown => "Unknown",
                        ConnectionType::__Nonexhaustive => "Future",
                    }
                    None => "connection type could not be obtained"
                    }
                }
                </p>
            </section>
        }
    }

    fn view_storage_info(&self, _ctx: &Context<Self>) -> Html {
        match &self.estimate {
            Some(estimate) => html! {
                <section class="section">
                    <div class="title">{"Storage Information"}</div>
                    <p>{"usage/quota"}</p>
                    <p>{format!("{} MB/{} MB ({:.0}%)", estimate.usage/1024/1024, estimate.quota/1024/1024, (estimate.usage as f64/estimate.quota as f64)*100.0 )}</p>
                </section>
            },
            None => html! {},
        }
    }

    fn process_estimate(&mut self, res: Result<JsValue, JsValue>) -> Result<(), JsError> {
        let val = res?;
        let est = serde_wasm_bindgen::from_value::<Estimate>(val)?;
        self.estimate = Some(est);
        Ok(())
    }
}

impl Component for InfoPage {
    type Message = Message;
    type Properties = Props;

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
                <NavBar/>
                <Notification/>
                { self.view_network_info(ctx) }
                { self.view_storage_info(ctx) }
            </>
        }
    }

    fn create(ctx: &Context<Self>) -> Self {
        let mut notifier = notifier::Notifier::dispatcher();

        ctx.link().send_future(async move {
            let storage_manager = web_sys::window().unwrap().navigator().storage();
            Message::GetEstimate(JsFuture::from(storage_manager.estimate().unwrap()).await)
        });
        ctx.link().send_future(async move {
            let storage_manager = web_sys::window().unwrap().navigator().storage();
            Message::GetPersisted(JsFuture::from(storage_manager.persisted().unwrap()).await)
        });
        ctx.link().send_future(async move {
            let storage_manager = web_sys::window().unwrap().navigator().storage();
            Message::GetPersist(JsFuture::from(storage_manager.persist().unwrap()).await)
        });

        Self {
            estimate: None,
            connection_type: match obtain_connection_type() {
                Ok(ct) => Some(ct),
                Err(e) => {
                    notifier.send(notifier::Request::Notify(notifier::Notification {
                        severity: notifier::NotificationSeverity::Error,
                        text: e.description,
                    }));
                    None
                }
            },
            notifier,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::GetEstimate(res) => match self.process_estimate(res) {
                Ok(()) => true,
                Err(_e) => false,
            },
            Message::GetPersisted(res) => {
                log::info!("persisted: {:?}", res);
                false
            }
            Message::GetPersist(res) => {
                log::info!("persist: {:?}", res);
                false
            }
        }
    }
}

fn obtain_connection_type() -> Result<ConnectionType, JsError> {
    Ok(web_sys::window()
        .ok_or("error getting window")?
        .navigator()
        .connection()?
        .type_())
}
