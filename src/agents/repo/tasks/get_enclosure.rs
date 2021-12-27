use crate::{agents::repo, objects::JsError};
use uuid::Uuid;
use wasm_bindgen::JsCast;
use web_sys::{IdbDatabase, IdbTransactionMode};

pub struct GetEnclosureTask {
    id: Uuid,
}

impl GetEnclosureTask {
    pub fn new_with_id(id: Uuid) -> Self {
        Self { id }
    }
}

impl repo::RepositoryTask for GetEnclosureTask {
    fn get_request(&mut self, db: &IdbDatabase) -> Result<Vec<web_sys::IdbRequest>, JsError> {
        let trans =
            self.create_transaction(&db, IdbTransactionMode::Readonly, &vec!["enclosures"])?;

        let os = trans.object_store("enclosures")?;
        Ok(vec![os.get(&serde_wasm_bindgen::to_value(&self.id)?)?])
    }

    fn set_response(
        &mut self,
        result: Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>,
    ) -> Result<Option<repo::Response>, JsError> {
        Ok(Some(repo::Response::Enclosure(result?.dyn_into()?)))
    }
}
