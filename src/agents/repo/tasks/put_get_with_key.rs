use crate::{agents::repo, objects::JsError};
use wasm_bindgen::JsValue;
use web_sys::{IdbDatabase, IdbTransactionMode};

pub struct PutGetWithKeyTask {
    kind: Kind,
    key: JsValue,
    value: Option<JsValue>,
}

pub enum Kind {
    // Items,
    // Channels,
    // Feeds,
    ConfigurationFetcher,
    ConfigurationUpdater,
}

impl PutGetWithKeyTask {
    pub fn new(kind: Kind, value: Option<JsValue>, key: JsValue) -> Self {
        Self { kind, key, value }
    }

    pub fn kind_str(&self) -> &str {
        match &self.kind {
            // Kind::Feeds => "feeds",
            // Kind::Channels => "channels",
            // Kind::Items => "items",
            Kind::ConfigurationFetcher | Kind::ConfigurationUpdater => "configuration",
        }
    }
}

impl repo::RepositoryTask for PutGetWithKeyTask {
    fn get_request(&mut self, db: &IdbDatabase) -> Result<Vec<web_sys::IdbRequest>, JsError> {
        let trans = self.create_transaction(
            &db,
            match self.value {
                Some(_) => IdbTransactionMode::Readwrite,
                None => IdbTransactionMode::Readonly,
            },
            &vec![self.kind_str()],
        )?;

        let os = trans.object_store(self.kind_str())?;

        if let Some(value) = &self.value {
            os.put_with_key(value, &self.key)?;
        }

        Ok(vec![os.get(&self.key)?])
    }

    fn set_response(
        &mut self,
        result: Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>,
    ) -> Result<Option<repo::Response>, JsError> {
        Ok(Some(match &self.kind {
            // Kind::Feeds => repo::Response::Feeds(serde_wasm_bindgen::from_value(result?)?),
            // Kind::Channels => repo::Response::Channels(serde_wasm_bindgen::from_value(result?)?),
            // Kind::Items => repo::Response::Items(serde_wasm_bindgen::from_value(result?)?),
            Kind::ConfigurationFetcher => {
                repo::Response::FetcherConfig(serde_wasm_bindgen::from_value(result?)?)
            }
            Kind::ConfigurationUpdater => {
                repo::Response::UpdaterConfig(serde_wasm_bindgen::from_value(result?)?)
            }
        }))
    }
}
