use crate::objects::JsError;
pub mod delete_enclosure;
pub mod download_started;
pub mod get_all;
pub mod get_keys;
pub mod open_db;
pub mod put_get_with_key;
pub mod store_enclosure;
pub mod sync_val;
use web_sys::{IdbRequest, IdbTransaction};

#[derive(Debug)]
pub enum Task {
    OpenDb(open_db::Task),
    GetAll(get_all::Task),
    PutGetWithKey(put_get_with_key::Task),
    StoreEnclosure(store_enclosure::Task),
    DownloadStarted(download_started::Task),
    DeleteEnclosure(delete_enclosure::Task),
    SyncVal(sync_val::Task),
    GetKeys(get_keys::Task),
}

impl Task {
    pub fn transaction_complete(&mut self) {
        match self {
            Task::GetAll(task) => task.transaction_complete(),
            Task::PutGetWithKey(task) => task.transaction_complete(),
            Task::StoreEnclosure(task) => task.transaction_complete(),
            Task::DownloadStarted(task) => task.transaction_complete(),
            Task::DeleteEnclosure(task) => task.transaction_complete(),
            Task::SyncVal(task) => task.transaction_complete(),
            Task::GetKeys(task) => task.transaction_complete(),
            Task::OpenDb(_) => {}
        }
    }
}

pub trait TaskProcessor<T> {
    fn process(&mut self, task: &mut T) -> Result<bool, JsError>;
}

fn request_ok(request: &IdbRequest) -> Result<(), JsError> {
    request
        .error()?
        .map_or(Ok::<(), JsError>(()), |e| Err(e.into()))
}

fn transaction_ok(transaction: &IdbTransaction) -> Result<(), JsError> {
    transaction
        .error()
        .map_or(Ok::<(), JsError>(()), |e| Err(e.into()))
}
