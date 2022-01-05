use crate::objects::JsError;
pub mod download_started;
pub mod get_all;
pub mod open_db;
pub mod put_get_with_key;
pub mod store_enclosure;
// pub use get_all;
// pub use open_db;
// pub use put_get_with_key;
// pub use store_enclosure;
use web_sys::{IdbRequest, IdbTransaction};

#[derive(Debug)]
pub enum Task {
    OpenDb(open_db::Task),
    GetAll(get_all::Task),
    PutGetWithKey(put_get_with_key::Task),
    StoreEnclosure(store_enclosure::Task),
    DownloadStarted(download_started::Task),
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
