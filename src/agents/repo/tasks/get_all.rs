use crate::{agents::repo, objects::JsError};
use web_sys::{IdbDatabase, IdbTransactionMode};

pub struct GetAllTask {
    kind: Kind,
}

pub enum Kind {
    Feed,
    Channel,
}

impl GetAllTask {
    pub fn new(kind: Kind) -> Self {
        Self { kind }
    }

    pub fn kind_str(&self) -> &str {
        match &self.kind {
            &Kind::Feed => "feeds",
            &Kind::Channel => "channels",
        }
    }
}

impl repo::RepositoryTask for GetAllTask {
    fn get_request(&mut self, db: &IdbDatabase) -> Result<Vec<web_sys::IdbRequest>, JsError> {
        let trans =
            self.create_transaction(&db, IdbTransactionMode::Readonly, &vec![self.kind_str()])?;

        let os = trans.object_store(self.kind_str())?;
        Ok(vec![os.get_all()?])
    }

    fn set_response(
        &mut self,
        result: Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>,
    ) -> Result<Option<repo::Response>, JsError> {
        Ok(Some(match &self.kind {
            &Kind::Feed => repo::Response::Feeds(serde_wasm_bindgen::from_value(result?)?),
            &Kind::Channel => repo::Response::Channels(serde_wasm_bindgen::from_value(result?)?),
        }))
    }
}
