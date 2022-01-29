use crate::{agents::repo, objects::JsError};
use chrono::{DateTime, FixedOffset};
use podcast_player_common::{channel_val::ChannelVal, item_val::ItemVal, Channel, FeedVal, Item};
use uuid::Uuid;
use wasm_bindgen::JsCast;

#[derive(Debug)]
pub struct Task {
    stage: Stage,
    value: Value,
    request: Option<web_sys::IdbRequest>,
    transaction: Option<web_sys::IdbTransaction>,
    object: Option<Object>,
}

#[derive(Debug)]
enum Stage {
    Init,
    WaitingForReadRequest,
    WaitingForWriteRequest,
    WaitingForTransaction,
    TransactionCompleted,
}

#[derive(Debug)]
pub enum Value {
    Feed(FeedVal),
    Channel(ChannelVal),
    Item(ItemVal),
}

impl Value {
    fn table_name(&self) -> &str {
        match &self {
            Self::Item(_) => "items",
            Self::Feed(_) => "feeds",
            Self::Channel(_) => "channels",
        }
    }

    fn id(&self) -> &Uuid {
        match &self {
            Self::Item(item_val) => &item_val.id,
            Self::Channel(channel_val) => &channel_val.id,
            Self::Feed(feed_val) => &feed_val.id,
        }
    }

    fn timestamp(&self) -> &DateTime<FixedOffset> {
        match &self {
            Self::Item(item_val) => &item_val.update_ts,
            Self::Channel(channel_val) => &channel_val.update_ts,
            Self::Feed(feed_val) => &feed_val.update_ts,
        }
    }

    fn as_ref(&self) -> &Self {
        &self
    }
}

impl Into<Object> for &Value {
    fn into(self) -> Object {
        match self {
            Value::Feed(feed_val) => Object::Feed(feed_val.clone()),
            Value::Channel(channel_val) => Object::Channel(Channel::from(channel_val)),
            Value::Item(item_val) => Object::Item(Item::from(item_val)),
        }
    }
}

impl TryInto<wasm_bindgen::JsValue> for &Object {
    type Error = JsError;

    fn try_into(self) -> Result<wasm_bindgen::JsValue, JsError> {
        match self {
            Object::Feed(feed) => serde_wasm_bindgen::to_value(feed).map_err(Into::into),
            Object::Channel(channel) => serde_wasm_bindgen::to_value(channel).map_err(Into::into),
            Object::Item(item) => serde_wasm_bindgen::to_value(item).map_err(Into::into),
        }
    }
}

#[derive(Debug)]
enum Object {
    Item(Item),
    Channel(Channel),
    Feed(FeedVal),
}

impl Object {
    fn get_val_update(&self) -> &DateTime<FixedOffset> {
        match &self {
            Self::Item(item) => item.get_val_update(),
            Self::Channel(channel) => channel.get_val_update(),
            Self::Feed(feed) => &feed.update_ts,
        }
    }

    fn as_ref(&self) -> &Self {
        &self
    }
}

impl Task {
    pub fn new(value: Value) -> Self {
        Self {
            stage: Stage::Init,
            value,
            request: None,
            transaction: None,
            object: None,
        }
    }

    pub fn transaction_complete(&mut self) {
        self.stage = Stage::TransactionCompleted;
    }
}

impl super::TaskProcessor<Task> for super::super::Repo {
    fn process(&mut self, task: &mut Task) -> Result<bool, JsError> {
        match task.stage {
            Stage::Init => {
                let db = self.db.as_ref().ok_or("db not set")?;
                let trans = db.transaction_with_str_sequence_and_mode(
                    &serde_wasm_bindgen::to_value(&vec![task.value.table_name()])?,
                    web_sys::IdbTransactionMode::Readwrite,
                )?;
                let os = trans.object_store(task.value.table_name())?;

                trans.set_onabort(Some(self.idb_closure_trans_abort.as_ref().unchecked_ref()));
                trans.set_onerror(Some(self.idb_closure_trans_error.as_ref().unchecked_ref()));
                trans.set_oncomplete(Some(
                    self.idb_closure_trans_complete.as_ref().unchecked_ref(),
                ));
                task.transaction = Some(trans);

                let request = os.get(&serde_wasm_bindgen::to_value(task.value.id())?)?;

                request.set_onsuccess(Some(self.idb_closure_success.as_ref().unchecked_ref()));
                request.set_onerror(Some(self.idb_closure_error.as_ref().unchecked_ref()));
                task.request = Some(request);

                task.stage = Stage::WaitingForReadRequest;
                Ok(false)
            }
            Stage::WaitingForReadRequest => {
                let request = task.request.as_ref().ok_or("request not set")?;

                match request.ready_state() {
                    web_sys::IdbRequestReadyState::Done => {
                        super::request_ok(request)?;

                        let existing_item: Option<Object> = match task.value {
                            Value::Channel(_) => serde_wasm_bindgen::from_value::<Option<Channel>>(
                                request.result()?,
                            )?
                            .map(|channel| Object::Channel(channel)),
                            Value::Feed(_) => serde_wasm_bindgen::from_value::<Option<FeedVal>>(
                                request.result()?,
                            )?
                            .map(|feed| Object::Feed(feed)),
                            Value::Item(_) => {
                                serde_wasm_bindgen::from_value::<Option<Item>>(request.result()?)?
                                    .map(|item| Object::Item(item))
                            }
                        };

                        match existing_item {
                            Some(existing_item) => {
                                if existing_item.get_val_update() < &task.value.timestamp() {
                                    let object: Object = task.value.as_ref().into();
                                    let request = task
                                        .transaction
                                        .as_ref()
                                        .ok_or("transaction not set")?
                                        .object_store(task.value.table_name())?
                                        .put_with_key(
                                            &object.as_ref().try_into()?,
                                            &serde_wasm_bindgen::to_value(task.value.id())?,
                                        )?;
                                    request.set_onsuccess(Some(
                                        self.idb_closure_success.as_ref().unchecked_ref(),
                                    ));
                                    request.set_onerror(Some(
                                        self.idb_closure_error.as_ref().unchecked_ref(),
                                    ));
                                    task.object = Some(object);
                                    task.request = Some(request);
                                    task.stage = Stage::WaitingForWriteRequest;

                                    Ok(false)
                                } else {
                                    task.stage = Stage::WaitingForTransaction;
                                    Ok(false)
                                }
                            }
                            None => {
                                let object: Object = task.value.as_ref().into();
                                let request = task
                                    .transaction
                                    .as_ref()
                                    .ok_or("transaction not set")?
                                    .object_store(task.value.table_name())?
                                    .add_with_key(
                                        &object.as_ref().try_into()?,
                                        &serde_wasm_bindgen::to_value(task.value.id())?,
                                    )?;
                                request.set_onsuccess(Some(
                                    self.idb_closure_success.as_ref().unchecked_ref(),
                                ));
                                request.set_onerror(Some(
                                    self.idb_closure_error.as_ref().unchecked_ref(),
                                ));
                                task.request = Some(request);
                                task.stage = Stage::WaitingForWriteRequest;
                                task.object = Some(object);

                                Ok(false)
                            }
                        }
                    }
                    _ => Ok(false),
                }
            }
            Stage::WaitingForWriteRequest => {
                let request = task.request.as_ref().ok_or("request not set")?;

                if request.ready_state() == web_sys::IdbRequestReadyState::Done {
                    super::request_ok(request)?;

                    task.stage = Stage::WaitingForTransaction;
                }

                Ok(false)
            }
            Stage::WaitingForTransaction => Ok(false),
            Stage::TransactionCompleted => {
                match &task.object {
                    Some(object) => {
                        let response = match object {
                            Object::Item(item) => repo::Response::UpdatedItem(item.clone()),
                            Object::Feed(feed) => repo::Response::UpdatedFeed(feed.clone()),
                            Object::Channel(channel) => {
                                repo::Response::UpdatedChannel(channel.clone())
                            }
                        };

                        for subscriber in &self.subscribers {
                            if subscriber.is_respondable() {
                                self.link.respond(*subscriber, response.clone());
                            }
                        }
                    }
                    None => {}
                }
                Ok(true)
            }
        }
    }
}
