use crate::{
    agents::notifier,
    components::{NavBar, Notification},
    objects::JsError,
    utils,
};
use serde::Deserialize;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::ConnectionType;
use yew::prelude::*;
use yew_agent::{Dispatched, Dispatcher};

/// TODO: move persist request to repository
pub struct InfoPage {
    estimate: Option<Estimate>,
    connection_type: Option<ConnectionType>,
    persisted: Option<bool>,
    notifier: Dispatcher<notifier::Notifier>,
}
pub enum Message {
    GetEstimate(Result<JsValue, JsValue>),
    GetPersisted(Result<JsValue, JsValue>),
    // GetPersist(Result<JsValue, JsValue>),
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
                <nav class="level">
                    <div class="level-item has-text-centered">
                        <div>
                            <p class="heading">{"connection type"}</p>
                            <p class="title">{match &self.connection_type {
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
                            }</p>
                        </div>
                    </div>
                </nav>
            </section>
        }
    }

    fn view_storage_info(&self, _ctx: &Context<Self>) -> Html {
        match (&self.estimate, &self.persisted) {
            (Some(estimate), Some(persisted)) => html! {
                <section class="section">
                    <div class="title">{"Storage Information"}</div>
                    <nav class="level">
                        <div class="level-item has-text-centered">
                            <div>
                                <p class="heading">{"usage"}</p>
                                <p class="title">{format!("{} MB", estimate.usage/1024/1024)}</p>
                            </div>
                        </div>
                        <div class="level-item has-text-centered">
                            <div>
                                <p class="heading">{"quota"}</p>
                                <p class="title">{format!("{} MB", estimate.quota/1024/1024)}</p>
                            </div>
                        </div>
                        <div class="level-item has-text-centered">
                            <div>
                                <p class="heading">{"percentage"}</p>
                                <p class="title">{format!("{:.0} %", (estimate.usage as f64/estimate.quota as f64)*100.0)}</p>
                            </div>
                        </div>
                        <div class="level-item has-text-centered">
                            <div>
                                <p class="heading">{"persisted"}</p>
                                <p class="title">{format!("{}", persisted)}</p>
                            </div>
                        </div>
                    </nav>
                </section>
            },
            (_, _) => html! {},
        }
    }

    fn process_estimate(&mut self, res: Result<JsValue, JsValue>) -> Result<(), JsError> {
        let val = res?;
        let est = serde_wasm_bindgen::from_value::<Estimate>(val)?;
        self.estimate = Some(est);
        Ok(())
    }

    fn process_persisted(&mut self, res: Result<JsValue, JsValue>) -> Result<(), JsError> {
        let val = res?;
        self.persisted = Some(serde_wasm_bindgen::from_value::<bool>(val)?);
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

        match obtain_estimate_future() {
            Ok(est) => ctx
                .link()
                .send_future(async move { Message::GetEstimate(est.await) }),
            Err(e) => notifier.send(notifier::Request::NotifyError(e)),
        };
        match obtain_persisted_future() {
            Ok(est) => ctx
                .link()
                .send_future(async move { Message::GetPersisted(est.await) }),
            Err(e) => notifier.send(notifier::Request::NotifyError(e)),
        };
        // ctx.link().send_future(async move {
        //     let storage_manager = web_sys::window().unwrap().navigator().storage();
        //     Message::GetPersist(JsFuture::from(storage_manager.persist().unwrap()).await)
        // });

        Self {
            estimate: None,
            connection_type: match utils::get_connection_type() {
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
            persisted: None,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::GetEstimate(res) => match self.process_estimate(res) {
                Ok(()) => true,
                Err(e) => {
                    self.notifier.send(notifier::Request::NotifyError(e));
                    false
                }
            },
            Message::GetPersisted(res) => match self.process_persisted(res) {
                Ok(()) => true,
                Err(e) => {
                    self.notifier.send(notifier::Request::NotifyError(e));
                    false
                }
            },
        }
    }
}

fn obtain_estimate_future() -> Result<JsFuture, JsError> {
    let storage_manager = web_sys::window()
        .ok_or("error getting storage manager")?
        .navigator()
        .storage();
    storage_manager
        .estimate()
        .map(JsFuture::from)
        .map_err(Into::into)
}

fn obtain_persisted_future() -> Result<JsFuture, JsError> {
    let storage_manager = web_sys::window()
        .ok_or("error getting storage manager")?
        .navigator()
        .storage();
    storage_manager
        .persisted()
        .map(JsFuture::from)
        .map_err(Into::into)
}
