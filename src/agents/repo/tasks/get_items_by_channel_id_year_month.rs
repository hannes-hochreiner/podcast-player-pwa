use crate::agents::repo;
use anyhow::{anyhow, Result};
use uuid::Uuid;
use web_sys::{IdbDatabase, IdbTransactionMode};

pub struct GetItemsByChannelIdYearMonthTask {
    channel_id: Uuid,
    year_month: String,
}

impl GetItemsByChannelIdYearMonthTask {
    pub fn new_with_channel_id_year_month(channel_id: Uuid, year_month: String) -> Self {
        Self {
            channel_id,
            year_month,
        }
    }
}

impl repo::RepositoryTask for GetItemsByChannelIdYearMonthTask {
    fn get_request(&mut self, db: &IdbDatabase) -> anyhow::Result<Vec<web_sys::IdbRequest>> {
        let trans = self.create_transaction(&db, IdbTransactionMode::Readonly, &vec!["items"])?;

        let os = trans
            .object_store("items")
            .map_err(|_e| anyhow!("error creating object store"))?;

        Ok(vec![os
            .index("channel_id_year_month")
            .map_err(|_e| anyhow!("error getting index"))?
            .get_all_with_key(
                &serde_wasm_bindgen::to_value(&vec![
                    self.channel_id.to_string(),
                    self.year_month.clone(),
                ])
                .map_err(|_e| anyhow!("error converting keys"))?,
            )
            .map_err(|_e| {
                anyhow!("error items with channel id and year month")
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
