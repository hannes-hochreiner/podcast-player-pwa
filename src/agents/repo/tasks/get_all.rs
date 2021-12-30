use crate::{agents::repo, objects::JsError};
use wasm_bindgen::JsValue;
use web_sys::{IdbDatabase, IdbTransactionMode};

pub struct GetAllTask {
    kind: Kind,
    key: Option<JsValue>,
}

pub enum Kind {
    Items,
    Channels,
    Feeds,
}

impl GetAllTask {
    pub fn new(kind: Kind, key: Option<JsValue>) -> Self {
        Self { kind, key }
    }

    pub fn kind_str(&self) -> &str {
        match &self.kind {
            &Kind::Feeds => "feeds",
            &Kind::Channels => "channels",
            &Kind::Items => "items",
        }
    }
}

impl repo::RepositoryTask for GetAllTask {
    fn get_request(&mut self, db: &IdbDatabase) -> Result<Vec<web_sys::IdbRequest>, JsError> {
        let trans =
            self.create_transaction(&db, IdbTransactionMode::Readonly, &vec![self.kind_str()])?;

        let os = trans.object_store(self.kind_str())?;
        Ok(vec![match &self.key {
            Some(key) => os.get_all_with_key(key)?,
            None => os.get_all()?,
        }])
    }

    fn set_response(
        &mut self,
        result: Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>,
    ) -> Result<Option<repo::Response>, JsError> {
        Ok(Some(match &self.kind {
            &Kind::Feeds => repo::Response::Feeds(serde_wasm_bindgen::from_value(result?)?),
            &Kind::Channels => repo::Response::Channels(serde_wasm_bindgen::from_value(result?)?),
            &Kind::Items => repo::Response::Items(serde_wasm_bindgen::from_value(result?)?),
        }))
    }
}
