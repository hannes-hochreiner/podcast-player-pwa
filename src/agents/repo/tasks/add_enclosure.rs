use crate::{
    agents::repo,
    objects::{DownloadStatus, Item},
};
use anyhow::{anyhow, Result};
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
    fn get_request(&mut self, db: &IdbDatabase) -> anyhow::Result<Vec<web_sys::IdbRequest>> {
        let trans = db
            .transaction_with_str_sequence_and_mode(
                &serde_wasm_bindgen::to_value(&vec!["items", "enclosures"]).unwrap(),
                IdbTransactionMode::Readwrite,
            )
            .unwrap();

        let os = trans.object_store("items").unwrap();
        self.transaction = Some(trans);

        Ok(vec![os
            .get(&serde_wasm_bindgen::to_value(&self.item_id).unwrap())
            .unwrap()])
    }

    fn set_response(
        &mut self,
        result: Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>,
    ) -> anyhow::Result<Option<repo::Response>> {
        match &self.transaction {
            Some(trans) => {
                let mut item: Item = serde_wasm_bindgen::from_value(
                    result.map_err(|_e| anyhow!("error getting item result"))?,
                )
                .map_err(|_e| anyhow!("error converting item result"))?;

                item.set_download_status(DownloadStatus::Ok(self.data.byte_length()));

                let item_os = trans.object_store("items").unwrap();

                item_os
                    .put_with_key(
                        &serde_wasm_bindgen::to_value(&item).unwrap(),
                        &serde_wasm_bindgen::to_value(&item.get_id()).unwrap(),
                    )
                    .unwrap();

                let enclosure_os = trans.object_store("enclosures").unwrap();

                enclosure_os
                    .put_with_key(
                        &self.data,
                        &serde_wasm_bindgen::to_value(&item.get_id()).unwrap(),
                    )
                    .unwrap();
                Ok(Some(repo::Response::Item(item)))
            }

            None => Err(anyhow!("error adding channel vals")),
        }
    }
}
