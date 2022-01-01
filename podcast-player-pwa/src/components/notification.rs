use crate::agents::notifier::{self, Notifier};
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

pub struct Notification {
    notifier: Box<dyn Bridge<Notifier>>,
    current_notification: Option<notifier::Notification>,
}

pub enum Message {
    NotifierResponse(notifier::Response),
    CloseNotification,
}

impl Component for Notification {
    type Message = Message;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            notifier: Notifier::bridge(ctx.link().callback(Message::NotifierResponse)),
            current_notification: None,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::NotifierResponse(resp) => match resp {
                notifier::Response::Notification(notification) => {
                    self.current_notification = notification;
                    true
                }
            },
            Message::CloseNotification => {
                self.notifier.send(notifier::Request::Dismiss);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match &self.current_notification {
            Some(notification) => {
                let color = match notification.severity {
                    notifier::NotificationSeverity::Error => "is-danger",
                    notifier::NotificationSeverity::Info => "is-primary",
                };
                let heading = match notification.severity {
                    notifier::NotificationSeverity::Error => "Error",
                    notifier::NotificationSeverity::Info => "Information",
                };
                html! {
                    <article class={classes!(format!("message {}", color))}>
                        <div class="message-header">
                            <p>{heading}</p>
                            <button class="delete" aria-label="delete" onclick={ctx.link().callback(|_| Message::CloseNotification)}></button>
                        </div>
                        <div class="message-body">
                            {notification.text.clone()}
                        </div>
                    </article>
                }
            }
            None => html! {},
        }
    }
}
