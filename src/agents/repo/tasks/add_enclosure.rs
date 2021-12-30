use crate::{
    agents::repo,
    objects::{DownloadStatus, Item, JsError},
};
use js_sys::ArrayBuffer;
use uuid::Uuid;
use web_sys::{IdbDatabase, IdbTransaction, IdbTransactionMode};

pub struct AddEnclosureTask {
    data: ArrayBuffer,
    item_id: Uuid,
    transaction: Option<IdbTransaction>,
}

impl AddEnclosureTask {
    pub fn new_with_item_id_data(item_id: Uuid, data: ArrayBuffer) -> Self {
        Self {
            item_id,
            data,
            transaction: None,
        }
    }
}

impl repo::RepositoryTask for AddEnclosureTask {
    fn get_request(&mut self, db: &IdbDatabase) -> Result<Vec<web_sys::IdbRequest>, JsError> {
        let trans = db.transaction_with_str_sequence_and_mode(
            &serde_wasm_bindgen::to_value(&vec!["items", "enclosures"])?,
            IdbTransactionMode::Readwrite,
        )?;

        let os = trans.object_store("items")?;
        self.transaction = Some(trans);

        Ok(vec![os.get(&serde_wasm_bindgen::to_value(&self.item_id)?)?])
    }

    fn set_response(
        &mut self,
        result: Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>,
    ) -> Result<Option<repo::Response>, JsError> {
        match &self.transaction {
            Some(trans) => {
                let mut item: Item = serde_wasm_bindgen::from_value(result?)?;

                item.set_download_status(DownloadStatus::Ok(self.data.byte_length()));

                let item_os = trans.object_store("items")?;

                item_os.put_with_key(
                    &serde_wasm_bindgen::to_value(&item)?,
                    &serde_wasm_bindgen::to_value(&item.get_id())?,
                )?;

                let enclosure_os = trans.object_store("enclosures")?;

                enclosure_os
                    .put_with_key(&self.data, &serde_wasm_bindgen::to_value(&item.get_id())?)?;
                Ok(Some(repo::Response::DownloadEnclosure(item)))
            }

            None => Err("error adding enclosure".into()),
        }
    }
}
