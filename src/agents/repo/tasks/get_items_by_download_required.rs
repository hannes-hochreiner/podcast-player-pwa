use crate::agents::repo;
use anyhow::{anyhow, Result};
use web_sys::{IdbDatabase, IdbTransactionMode};

pub struct GetItemsByDownloadRequiredTask {}

impl GetItemsByDownloadRequiredTask {
    pub fn new() -> Self {
        Self {}
    }
}

impl repo::RepositoryTask for GetItemsByDownloadRequiredTask {
    fn get_request(&mut self, db: &IdbDatabase) -> anyhow::Result<Vec<web_sys::IdbRequest>> {
        let trans = self.create_transaction(&db, IdbTransactionMode::Readonly, &vec!["items"])?;

        let os = trans
            .object_store("items")
            .map_err(|_e| anyhow!("error creating object store"))?;

        Ok(vec![os
            .index("download_required")
            .map_err(|_e| anyhow!("error getting index"))?
            .get_all_with_key(
                &serde_wasm_bindgen::to_value(&vec![String::from("true")])
                    .map_err(|_e| anyhow!("error converting keys"))?,
            )
            .map_err(|_e| {
                anyhow!("error items by download required")
            })?])
    }

    fn set_response(
        &mut self,
        result: Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>,
    ) -> anyhow::Result<Option<repo::Response>> {
        Ok(Some(repo::Response::Items(
            serde_wasm_bindgen::from_value(
                result.map_err(|_e| anyhow!("error getting item result"))?,
            )
            .map_err(|_e| anyhow!("error converting item result"))?,
        )))
    }
}