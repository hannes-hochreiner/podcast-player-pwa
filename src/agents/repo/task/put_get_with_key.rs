use crate::{agents::repo, objects::JsError};
use wasm_bindgen::{JsCast, JsValue};
use yew_agent::HandlerId;

#[derive(Debug)]
pub struct Task {
    stage: Stage,
    kind: Kind,
    key: JsValue,
    value: Option<JsValue>,
    request: Option<web_sys::IdbRequest>,
    transaction: Option<web_sys::IdbTransaction>,
    handler_id: Option<HandlerId>,
}

#[derive(Debug)]
enum Stage {
    Init,
    WaitingForWriteRequest,
    WaitingForReadRequest,
    WaitingForTransaction,
    TransactionCompleted,
}

#[derive(Debug)]
pub enum Kind {
    Feed,
    Channel,
    Item,
    Configuration,
    Enclosure,
}

impl Kind {
    fn table_name(&self) -> &str {
        match &self {
            Self::Configuration => "configuration",
            Self::Item => "items",
            Self::Feed => "feeds",
            Self::Channel => "channels",
            Self::Enclosure => "enclosures",
        }
    }
}

impl Task {
    pub fn new(
        handler_id: Option<HandlerId>,
        kind: Kind,
        key: JsValue,
        value: Option<JsValue>,
    ) -> Self {
        Self {
            stage: Stage::Init,
            key,
            kind,
            value,
            request: None,
            transaction: None,
            handler_id: None,
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
                if let Some(db) = &self.db {
                    let trans = db.transaction_with_str_sequence_and_mode(
                        &serde_wasm_bindgen::to_value(&vec![task.kind.table_name()])?,
                        match &task.value {
                            Some(_) => web_sys::IdbTransactionMode::Readwrite,
                            None => web_sys::IdbTransactionMode::Readonly,
                        },
                    )?;
                    let os = trans.object_store(task.kind.table_name())?;

                    trans.set_onabort(Some(self.idb_closure_trans_abort.as_ref().unchecked_ref()));
                    trans.set_onerror(Some(self.idb_closure_trans_error.as_ref().unchecked_ref()));
                    trans.set_oncomplete(Some(
                        self.idb_closure_trans_complete.as_ref().unchecked_ref(),
                    ));
                    task.transaction = Some(trans);

                    match &task.value {
                        Some(value) => {
                            let request = os.put_with_key(value, &task.key)?;

                            request.set_onsuccess(Some(
                                self.idb_closure_success.as_ref().unchecked_ref(),
                            ));
                            request
                                .set_onerror(Some(self.idb_closure_error.as_ref().unchecked_ref()));
                            task.request = Some(request);

                            task.stage = Stage::WaitingForWriteRequest;
                            Ok(false)
                        }
                        None => {
                            let request = os.get(&task.key)?;

                            request.set_onsuccess(Some(
                                self.idb_closure_success.as_ref().unchecked_ref(),
                            ));
                            request
                                .set_onerror(Some(self.idb_closure_error.as_ref().unchecked_ref()));
                            task.request = Some(request);

                            task.stage = Stage::WaitingForReadRequest;
                            Ok(false)
                        }
                    }
                } else {
                    Err(JsError::from_str("database not set"))
                }
            }
            Stage::WaitingForWriteRequest => match &task.request {
                Some(request) => match request.ready_state() {
                    web_sys::IdbRequestReadyState::Done => {
                        super::request_ok(request)?;

                        let request = task
                            .transaction
                            .as_ref()
                            .ok_or("transaction not set")?
                            .object_store(task.kind.table_name())?
                            .get(&task.key)?;

                        request
                            .set_onsuccess(Some(self.idb_closure_success.as_ref().unchecked_ref()));
                        request.set_onerror(Some(self.idb_closure_error.as_ref().unchecked_ref()));
                        task.request = Some(request);
                        task.stage = Stage::WaitingForReadRequest;

                        Ok(false)
                    }
                    _ => Ok(false),
                },
                None => Err(JsError::from_str("no request set")),
            },
            Stage::WaitingForReadRequest => {
                let request = task.request.as_ref().ok_or("request not set")?;

                if request.ready_state() == web_sys::IdbRequestReadyState::Done {
                    super::request_ok(request)?;

                    task.stage = Stage::WaitingForTransaction;
                }

                Ok(false)
            }
            Stage::WaitingForTransaction => Ok(false),
            Stage::TransactionCompleted => {
                let result = task.request.as_ref().ok_or("request not set")?.result()?;
                let response = match &task.kind {
                    Kind::Item => Ok(repo::Response::UpdatedItem(serde_wasm_bindgen::from_value(
                        result.clone(),
                    )?)),
                    Kind::Feed => Ok(repo::Response::UpdatedFeed(serde_wasm_bindgen::from_value(
                        result.clone(),
                    )?)),
                    Kind::Channel => Ok(repo::Response::UpdatedChannel(
                        serde_wasm_bindgen::from_value(result.clone())?,
                    )),
                    Kind::Configuration => {
                        let key: String = serde_wasm_bindgen::from_value(task.key.clone())?;

                        match &*key {
                            "fetcher" => Ok(repo::Response::FetcherConfig(
                                serde_wasm_bindgen::from_value(result.clone())?,
                            )),
                            "updater" => Ok(repo::Response::UpdaterConfig(
                                serde_wasm_bindgen::from_value(result.clone())?,
                            )),
                            _ => Err(JsError::from_str("unknown configuration requested")),
                        }
                    }
                    Kind::Enclosure => Ok(repo::Response::Enclosure(result.dyn_into()?)),
                }?;

                match task.handler_id {
                    Some(handler_id) => {
                        self.link.respond(handler_id, response);
                    }
                    None => {
                        for subscriber in &self.subscribers {
                            self.link.respond(*subscriber, response.clone());
                        }
                    }
                }

                Ok(true)
            }
        }
    }
}
