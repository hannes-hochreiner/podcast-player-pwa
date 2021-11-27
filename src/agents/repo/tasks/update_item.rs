use crate::agents::repo;
use crate::objects::item::Item;
use anyhow::{anyhow, Result};
use web_sys::{IdbDatabase, IdbTransactionMode};

pub struct UpdateItemTask {
    item: Item,
}

impl UpdateItemTask {
    pub fn new_with_item(item: Item) -> Self {
        Self { item }
    }
}

impl repo::RepositoryTask for UpdateItemTask {
    fn get_request(&mut self, db: &IdbDatabase) -> anyhow::Result<Vec<web_sys::IdbRequest>> {
        let transaction = db
            .transaction_with_str_sequence_and_mode(
                &serde_wasm_bindgen::to_value(&vec!["items"]).unwrap(),
                IdbTransactionMode::Readwrite,
            )
            .unwrap();
        let item_os = transaction.object_store("items").unwrap();
        item_os
            .put_with_key(
                &serde_wasm_bindgen::to_value(&self.item).unwrap(),
                &serde_wasm_bindgen::to_value(&self.item.get_id()).unwrap(),
            )
            .unwrap();
        Ok(vec![item_os
            .get(&serde_wasm_bindgen::to_value(&self.item.get_id()).unwrap())
            .unwrap()])
    }

    fn set_response(
        &mut self,
        result: Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>,
    ) -> anyhow::Result<Option<repo::Response>> {
        Ok(Some(repo::Response::Item(
            serde_wasm_bindgen::from_value(
                result.map_err(|_e| anyhow!("error getting item result"))?,
            )
            .map_err(|_e| anyhow!("error converting item result"))?,
        )))
    }
}
