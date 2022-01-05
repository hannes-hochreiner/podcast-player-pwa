use crate::agents::repo::Response;
use podcast_player_common::{DownloadStatus, Item};
use wasm_bindgen::JsCast;
use web_sys::IdbTransaction;

#[derive(Debug)]
pub struct Task {
    stage: Stage,
    item: Item,
    enclosure_request: Option<web_sys::IdbRequest>,
    item_request: Option<web_sys::IdbRequest>,
    transaction: Option<IdbTransaction>,
}

#[derive(Debug)]
enum Stage {
    Init,
    WaitingForRequest,
    WaitingForTransaction,
    TransactionCompleted,
}

impl Task {
    pub fn new(item: Item) -> Self {
        Self {
            stage: Stage::Init,
            enclosure_request: None,
            item_request: None,
            item,
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
                let db = self.db.as_ref().ok_or("db not set")?;
                let trans = db.transaction_with_str_sequence_and_mode(
                    &serde_wasm_bindgen::to_value(&vec!["items", "enclosures"])?,
                    web_sys::IdbTransactionMode::Readwrite,
                )?;
                let enclosure_request = trans
                    .object_store("enclosures")?
                    .delete(&serde_wasm_bindgen::to_value(&task.item.get_id())?)?;
                task.item.set_download(false);
                task.item.set_download_status(DownloadStatus::NotRequested);
                let item_request = trans.object_store("items")?.put_with_key(
                    &serde_wasm_bindgen::to_value(&task.item)?,
                    &serde_wasm_bindgen::to_value(&task.item.get_id())?,
                )?;

                trans.set_onabort(Some(self.idb_closure_trans_abort.as_ref().unchecked_ref()));
                trans.set_onerror(Some(self.idb_closure_trans_error.as_ref().unchecked_ref()));
                trans.set_oncomplete(Some(
                    self.idb_closure_trans_complete.as_ref().unchecked_ref(),
                ));
                task.transaction = Some(trans);

                item_request.set_onsuccess(Some(self.idb_closure_success.as_ref().unchecked_ref()));
                item_request.set_onerror(Some(self.idb_closure_error.as_ref().unchecked_ref()));
                task.item_request = Some(item_request);

                enclosure_request
                    .set_onsuccess(Some(self.idb_closure_success.as_ref().unchecked_ref()));
                enclosure_request
                    .set_onerror(Some(self.idb_closure_error.as_ref().unchecked_ref()));
                task.enclosure_request = Some(enclosure_request);

                task.stage = Stage::WaitingForRequest;
                Ok(false)
            }
            Stage::WaitingForRequest => {
                let enclosure_request = task
                    .enclosure_request
                    .as_ref()
                    .ok_or("enclosure request not set")?;

                let item_request = task.item_request.as_ref().ok_or("item request not set")?;

                if (enclosure_request.ready_state() == web_sys::IdbRequestReadyState::Done)
                    & (item_request.ready_state() == web_sys::IdbRequestReadyState::Done)
                {
                    super::request_ok(enclosure_request)?;
                    super::request_ok(item_request)?;

                    task.stage = Stage::WaitingForTransaction;
                }

                Ok(false)
            }
            Stage::WaitingForTransaction => Ok(false),
            Stage::TransactionCompleted => {
                super::transaction_ok(task.transaction.as_ref().ok_or("transaction not set")?)?;
                let item = task.item.clone();

                for subscriber in &self.subscribers {
                    self.link
                        .respond(*subscriber, Response::UpdatedItem(item.clone()));
                }

                Ok(true)
            }
        }
    }
}
