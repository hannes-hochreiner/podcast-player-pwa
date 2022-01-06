use crate::agents::repo;
use js_sys::ArrayBuffer;
use podcast_player_common::{item_meta::DownloadStatus, Item};
use uuid::Uuid;
use wasm_bindgen::JsCast;
use web_sys::{IdbRequest, IdbRequestReadyState, IdbTransaction, IdbTransactionMode};

#[derive(Debug)]
pub struct Task {
    stage: Stage,
    item_id: Uuid,
    data: ArrayBuffer,
    item_read_request: Option<IdbRequest>,
    item_write_request: Option<IdbRequest>,
    data_write_request: Option<IdbRequest>,
    item: Option<Item>,
    transaction: Option<IdbTransaction>,
}

#[derive(Debug)]
enum Stage {
    Init,
    WaitingForIdbReadRequest,
    WaitingForIdbWriteRequests,
    WaitingForTransaction,
    TransactionCompleted,
}

impl Task {
    pub fn new(item_id: Uuid, data: ArrayBuffer) -> Self {
        Self {
            stage: Stage::Init,
            item_id,
            data,
            item_read_request: None,
            data_write_request: None,
            item_write_request: None,
            item: None,
            transaction: None,
        }
    }

    pub fn transaction_complete(&mut self) {
        self.stage = Stage::TransactionCompleted;
    }
}

impl super::TaskProcessor<Task> for super::super::Repo {
    fn process(&mut self, task: &mut Task) -> Result<bool, crate::objects::JsError> {
        match &task.stage {
            Stage::Init => {
                let trans = self
                    .db
                    .as_ref()
                    .ok_or("db not set")?
                    .transaction_with_str_sequence_and_mode(
                        &serde_wasm_bindgen::to_value(&vec!["items", "enclosures"])?,
                        IdbTransactionMode::Readwrite,
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
                task.item_read_request = Some(request);

                task.stage = Stage::WaitingForIdbReadRequest;
                Ok(false)
            }
            Stage::WaitingForIdbReadRequest => {
                let request = task
                    .item_read_request
                    .as_ref()
                    .ok_or("idb request not set")?;

                match request.ready_state() {
                    IdbRequestReadyState::Done => {
                        // TODO: check download size
                        let trans = request.transaction().ok_or("transaction not set")?;

                        let mut item: Item = serde_wasm_bindgen::from_value(request.result()?)?;

                        item.set_download_status(DownloadStatus::Ok(task.data.byte_length()));

                        let item_os = trans.object_store("items")?;
                        let item_write_request = item_os.put_with_key(
                            &serde_wasm_bindgen::to_value(&item)?,
                            &serde_wasm_bindgen::to_value(&item.get_id())?,
                        )?;

                        let enclosure_os = trans.object_store("enclosures")?;
                        let data_write_request = enclosure_os.put_with_key(
                            &task.data,
                            &serde_wasm_bindgen::to_value(&item.get_id())?,
                        )?;

                        item_write_request
                            .set_onsuccess(Some(self.idb_closure_success.as_ref().unchecked_ref()));
                        item_write_request
                            .set_onerror(Some(self.idb_closure_error.as_ref().unchecked_ref()));

                        data_write_request
                            .set_onsuccess(Some(self.idb_closure_success.as_ref().unchecked_ref()));
                        data_write_request
                            .set_onerror(Some(self.idb_closure_error.as_ref().unchecked_ref()));

                        task.item_write_request = Some(item_write_request);
                        task.data_write_request = Some(data_write_request);
                        task.item = Some(item);
                        task.stage = Stage::WaitingForIdbWriteRequests;

                        Ok(false)
                    }
                    _ => Ok(false),
                }
            }
            Stage::WaitingForIdbWriteRequests => {
                let item_request = task
                    .item_write_request
                    .as_ref()
                    .ok_or("item write request not set")?;
                let data_request = task
                    .data_write_request
                    .as_ref()
                    .ok_or("data write request not set")?;

                match (item_request.ready_state(), data_request.ready_state()) {
                    (IdbRequestReadyState::Done, IdbRequestReadyState::Done) => {
                        super::request_ok(item_request)?;
                        super::request_ok(data_request)?;
                        task.stage = Stage::WaitingForTransaction;
                    }
                    (_, _) => {}
                }

                Ok(false)
            }
            Stage::WaitingForTransaction => Ok(false),
            Stage::TransactionCompleted => {
                super::transaction_ok(task.transaction.as_ref().ok_or("transaction not set")?)?;
                let item = task.item.as_ref().ok_or("item not set")?;

                for subscriber in &self.subscribers {
                    if subscriber.is_respondable() {
                        self.link
                            .respond(*subscriber, repo::Response::UpdatedItem(item.clone()));
                    }
                }

                Ok(true)
            }
        }
    }
}
