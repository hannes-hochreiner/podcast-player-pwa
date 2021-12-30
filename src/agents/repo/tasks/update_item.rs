use crate::{
    agents::repo,
    objects::{Item, JsError},
};
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
    fn get_request(&mut self, db: &IdbDatabase) -> Result<Vec<web_sys::IdbRequest>, JsError> {
        let transaction = db.transaction_with_str_sequence_and_mode(
            &serde_wasm_bindgen::to_value(&vec!["items"])?,
            IdbTransactionMode::Readwrite,
        )?;
        let item_os = transaction.object_store("items")?;
        item_os.put_with_key(
            &serde_wasm_bindgen::to_value(&self.item)?,
            &serde_wasm_bindgen::to_value(&self.item.get_id())?,
        )?;
        Ok(vec![item_os.get(&serde_wasm_bindgen::to_value(
            &self.item.get_id(),
        )?)?])
    }

    fn set_response(
        &mut self,
        result: Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>,
    ) -> Result<Option<repo::Response>, JsError> {
        Ok(Some(repo::Response::UpdateItem(
            serde_wasm_bindgen::from_value(result?)?,
        )))
    }
}
