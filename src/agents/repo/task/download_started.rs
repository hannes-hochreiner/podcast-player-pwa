use crate::{agents::repo, objects::JsError};
use podcast_player_common::{DownloadStatus, Item};
use uuid::Uuid;
use wasm_bindgen::JsCast;

#[derive(Debug)]
pub struct Task {
    stage: Stage,
    item_id: Uuid,
    request: Option<web_sys::IdbRequest>,
    transaction: Option<web_sys::IdbTransaction>,
    item: Option<Item>,
}

#[derive(Debug)]
enum Stage {
    Init,
    WaitingForReadRequest,
    WaitingForWriteRequest,
    WaitingForTransaction,
    TransactionCompleted,
}

impl Task {
    pub fn new(item_id: Uuid) -> Self {
        Self {
            stage: Stage::Init,
            item_id,
            request: None,
            transaction: None,
            item: None,
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
                        &serde_wasm_bindgen::to_value(&vec!["items"])?,
                        web_sys::IdbTransactionMode::Readwrite,
                    )?;
                    let os = trans.object_store("items")?;

                    trans.set_onabort(Some(self.idb_closure_trans_abort.as_ref().unchecked_ref()));
                    trans.set_onerror(Some(self.idb_closure_trans_error.as_ref().unchecked_ref()));
                    trans.set_oncomplete(Some(
                        self.idb_closure_trans_complete.as_ref().unchecked_ref(),
                    ));
                    task.transaction = Some(trans);

                    let request = os.get(&serde_wasm_bindgen::to_value(&task.item_id)?)?;

                    request.set_onsuccess(Some(self.idb_closure_success.as_ref().unchecked_ref()));
                    request.set_onerror(Some(self.idb_closure_error.as_ref().unchecked_ref()));
                    task.request = Some(request);
                    task.stage = Stage::WaitingForReadRequest;

                    Ok(false)
                } else {
                    Err(JsError::from_str("database not set"))
                }
            }
            Stage::WaitingForReadRequest => {
                let request = task.request.as_ref().ok_or("request not set")?;

                if request.ready_state() == web_sys::IdbRequestReadyState::Done {
                    super::request_ok(request)?;

                    let mut item: Item = serde_wasm_bindgen::from_value(request.result()?)?;

                    item.set_download_status(DownloadStatus::InProgress);

                    let request = task
                        .transaction
                        .as_ref()
                        .ok_or("transaction not set")?
                        .object_store("items")?
                        .put_with_key(
                            &serde_wasm_bindgen::to_value(&item)?,
                            &serde_wasm_bindgen::to_value(&item.get_id())?,
                        )?;

                    request.set_onsuccess(Some(self.idb_closure_success.as_ref().unchecked_ref()));
                    request.set_onerror(Some(self.idb_closure_error.as_ref().unchecked_ref()));
                    task.request = Some(request);
                    task.stage = Stage::WaitingForWriteRequest;
                    task.item = Some(item);
                }

                Ok(false)
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
                let response =
                    repo::Response::UpdatedItem(task.item.as_ref().ok_or("item not set")?.clone());

                for subscriber in &self.subscribers {
                    self.link.respond(*subscriber, response.clone());
                }

                Ok(true)
            }
        }
    }
}
