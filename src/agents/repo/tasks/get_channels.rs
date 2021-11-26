use crate::agents::repo;
use anyhow::{anyhow, Result};
use web_sys::{IdbDatabase, IdbTransactionMode};

pub struct GetChannelsTask {}

impl GetChannelsTask {
    pub fn new() -> Self {
        Self {}
    }
}

impl repo::RepositoryTask for GetChannelsTask {
    fn get_request(&mut self, db: &IdbDatabase) -> Result<Vec<web_sys::IdbRequest>> {
        let trans =
            self.create_transaction(&db, IdbTransactionMode::Readonly, &vec!["channels"])?;

        let os = trans
            .object_store("channels")
            .map_err(|_e| anyhow!("error creating object store"))?;
        Ok(vec![os
            .get_all()
            .map_err(|_e| anyhow!("error getting all channels"))?])
    }

    fn set_response(
        &mut self,
        result: Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>,
    ) -> Result<Option<repo::Response>> {
        Ok(Some(repo::Response::Channels(
            serde_wasm_bindgen::from_value(
                result.map_err(|_e| anyhow!("error getting channel result"))?,
            )
            .map_err(|_e| anyhow!("error converting channel result"))?,
        )))
    }
}
