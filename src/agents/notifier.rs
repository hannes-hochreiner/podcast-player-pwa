use std::collections::HashSet;
use yew_agent::{Agent, AgentLink, Context, HandlerId};

use crate::objects::JsError;

pub struct Notifier {
    subscribers: HashSet<HandlerId>,
    link: AgentLink<Self>,
    notifications: Vec<Notification>,
}

#[derive(Debug, Clone)]
pub enum NotificationSeverity {
    Error,
    Info,
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub text: String,
    pub severity: NotificationSeverity,
}

pub enum Request {
    Notify(Notification),
    NotifyError(JsError),
    Dismiss,
}

pub enum Response {
    Notification(Option<Notification>),
}

impl Notifier {
    fn notify_subscribed(&self) {
        for subscriber in &self.subscribers {
            if subscriber.is_respondable() {
                self.link.respond(
                    *subscriber,
                    Response::Notification(match self.notifications.is_empty() {
                        true => None,
                        false => Some(self.notifications[0].clone()),
                    }),
                );
            }
        }
    }
}

impl Agent for Notifier {
    type Reach = Context<Self>;
    type Message = ();
    type Input = Request;
    type Output = Response;

    fn create(link: AgentLink<Self>) -> Self {
        Self {
            link,
            subscribers: HashSet::new(),
            notifications: Vec::new(),
        }
    }

    fn update(&mut self, _msg: Self::Message) {
        unimplemented!()
    }

    fn handle_input(&mut self, msg: Self::Input, _id: HandlerId) {
        match msg {
            Request::Notify(notification) => {
                match notification.severity {
                    NotificationSeverity::Error => log::error!("{}", notification.text),
                    NotificationSeverity::Info => log::info!("{}", notification.text),
                }
                self.notifications.push(notification)
            }
            Request::NotifyError(err) => {
                log::error!("{}", err);
                self.notifications.push(Notification {
                    severity: NotificationSeverity::Error,
                    text: err.description,
                })
            }
            Request::Dismiss => {
                self.notifications.remove(0);
                self.notify_subscribed();
            }
        }
        self.notify_subscribed();
    }

    fn connected(&mut self, id: HandlerId) {
        self.subscribers.insert(id);
        self.notify_subscribed();
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
    }
}
