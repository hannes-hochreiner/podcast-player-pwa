use crate::agents::repo;
use anyhow::Result;
use uuid::Uuid;
use web_sys::IdbDatabase;

pub struct DownloadEnclosureTask {
    item_id: Uuid,
}

impl DownloadEnclosureTask {
    pub fn new(item_id: Uuid) -> Self {
        Self { item_id }
    }
}

impl repo::RepositoryTask for DownloadEnclosureTask {
    fn get_request(&mut self, _db: &IdbDatabase) -> Result<Vec<web_sys::IdbRequest>> {
        unimplemented!()
    }

    fn set_response(
        &mut self,
        _result: Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>,
    ) -> Result<Option<repo::Response>> {
        unimplemented!()
    }
}
