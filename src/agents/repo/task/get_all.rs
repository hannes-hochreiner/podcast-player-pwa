use crate::{
    agents::{fetcher, repo},
    objects::JsError,
};

use podcast_player_common::Item;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::IdbTransaction;
use yew_agent::HandlerId;

#[derive(Debug)]
pub struct Task {
    stage: Stage,
    key: Option<JsValue>,
    kind: Kind,
    index: Option<String>,
    request: Option<web_sys::IdbRequest>,
    handler_id: Option<HandlerId>,
    transaction: Option<IdbTransaction>,
}

#[derive(Debug)]
enum Stage {
    Init,
    WaitingForRequest,
    WaitingForTransaction,
    TransactionCompleted,
}

#[derive(Debug)]
pub enum Kind {
    Feed,
    Channel,
    Item,
    ItemDownloadRequired,
}

impl Kind {
    fn table_name(&self) -> &str {
        match &self {
            Self::Item => "items",
            Self::Feed => "feeds",
            Self::Channel => "channels",
            Self::ItemDownloadRequired => "items",
        }
    }
}

impl Task {
    pub fn new(
        handler_id: Option<HandlerId>,
        kind: Kind,
        key: Option<JsValue>,
        index: Option<String>,
    ) -> Self {
        Self {
            stage: Stage::Init,
            request: None,
            index,
            key,
            kind,
            handler_id,
            transaction: None,
        }
    }

    pub fn transaction_complete(&mut self) {
        self.stage = Stage::TransactionCompleted;
    }
}

impl super::TaskProcessor<Task> for super::super::Repo {
    fn process(&mut self, task: &mut Task) -> Result<bool, crate::objects::JsError> {
        match task.stage {
            Stage::Init => {
                if let Some(db) = &self.db {
                    let trans = db.transaction_with_str_sequence_and_mode(
                        &serde_wasm_bindgen::to_value(&vec![task.kind.table_name()])?,
                        web_sys::IdbTransactionMode::Readonly,
                    )?;
                    let os = trans.object_store(task.kind.table_name())?;
                    let request = match (&task.index, &task.key) {
                        (Some(index), Some(key)) => os.index(index)?.get_all_with_key(key)?,
                        (None, Some(key)) => os.get_all_with_key(key)?,
                        (Some(index), None) => os.index(index)?.get_all()?,
                        (None, None) => os.get_all()?,
                    };

                    trans.set_onabort(Some(self.idb_closure_trans_abort.as_ref().unchecked_ref()));
                    trans.set_onerror(Some(self.idb_closure_trans_error.as_ref().unchecked_ref()));
                    trans.set_oncomplete(Some(
                        self.idb_closure_trans_complete.as_ref().unchecked_ref(),
                    ));
                    task.transaction = Some(trans);

                    request.set_onsuccess(Some(self.idb_closure_success.as_ref().unchecked_ref()));
                    request.set_onerror(Some(self.idb_closure_error.as_ref().unchecked_ref()));
                    task.request = Some(request);

                    task.stage = Stage::WaitingForRequest;
                    Ok(false)
                } else {
                    Err(JsError::from_str("database not set"))
                }
            }
            Stage::WaitingForRequest => {
                let request = task.request.as_ref().ok_or("request not set")?;

                if request.ready_state() == web_sys::IdbRequestReadyState::Done {
                    super::request_ok(request)?;

                    task.stage = Stage::WaitingForTransaction;
                }

                Ok(false)
            }
            Stage::WaitingForTransaction => Ok(false),
            Stage::TransactionCompleted => {
                super::transaction_ok(task.transaction.as_ref().ok_or("transaction not set")?)?;
                let result = task.request.as_ref().ok_or("request not set")?.result()?;

                match &task.kind {
                    Kind::Item => {
                        self.link.respond(
                            task.handler_id.ok_or("handler id not set")?,
                            repo::Response::Items(serde_wasm_bindgen::from_value(result)?),
                        );
                    }
                    Kind::Channel => {
                        self.link.respond(
                            task.handler_id.ok_or("handler id not set")?,
                            repo::Response::Channels(serde_wasm_bindgen::from_value(result)?),
                        );
                    }
                    Kind::Feed => {
                        self.link.respond(
                            task.handler_id.ok_or("handler id not set")?,
                            repo::Response::Feeds(serde_wasm_bindgen::from_value(result)?),
                        );
                    }
                    Kind::ItemDownloadRequired => {
                        let items: Vec<Item> = serde_wasm_bindgen::from_value(result)?;

                        for item in items {
                            self.fetcher
                                .send(fetcher::Request::PullDownload(item.get_id()));
                        }
                    }
                }
                Ok(true)
            }
        }
    }
}
