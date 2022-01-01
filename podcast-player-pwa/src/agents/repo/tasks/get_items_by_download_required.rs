use crate::{agents::repo, objects::JsError};
use web_sys::{IdbDatabase, IdbTransactionMode};

pub struct GetItemsByDownloadRequiredTask {}

impl GetItemsByDownloadRequiredTask {
    pub fn new() -> Self {
        Self {}
    }
}

impl repo::RepositoryTask for GetItemsByDownloadRequiredTask {
    fn get_request(&mut self, db: &IdbDatabase) -> Result<Vec<web_sys::IdbRequest>, JsError> {
        let trans = self.create_transaction(&db, IdbTransactionMode::Readonly, &vec!["items"])?;

        let os = trans.object_store("items")?;

        Ok(vec![os.index("download_required")?.get_all_with_key(
            &serde_wasm_bindgen::to_value(&vec![String::from("true")])?,
        )?])
    }

    fn set_response(
        &mut self,
        result: Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>,
    ) -> Result<Option<repo::Response>, JsError> {
        Ok(Some(repo::Response::Items(serde_wasm_bindgen::from_value(
            result?,
        )?)))
    }
}
