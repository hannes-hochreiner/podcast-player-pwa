use crate::{agents::repo, objects::JsError};
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
    fn get_request(&mut self, db: &IdbDatabase) -> Result<Vec<web_sys::IdbRequest>, JsError> {
        let trans = self.create_transaction(&db, IdbTransactionMode::Readonly, &vec!["items"])?;

        let os = trans.object_store("items")?;

        Ok(vec![os.index("channel_id_year_month")?.get_all_with_key(
            &serde_wasm_bindgen::to_value(&vec![
                self.channel_id.to_string(),
                self.year_month.clone(),
            ])?,
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
