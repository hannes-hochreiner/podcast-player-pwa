use crate::{
    agents::repo,
    objects::{FetcherConfig, JsError},
};
use web_sys::{IdbDatabase, IdbTransactionMode};

pub struct GetFetcherConfTask {
    fct: Option<FetcherConfig>,
}

impl GetFetcherConfTask {
    pub fn new_with_option(fct: Option<FetcherConfig>) -> Self {
        Self { fct }
    }
}

impl repo::RepositoryTask for GetFetcherConfTask {
    fn get_request(&mut self, db: &IdbDatabase) -> Result<Vec<web_sys::IdbRequest>, JsError> {
        let trans = self.create_transaction(
            &db,
            match &self.fct {
                Some(_) => IdbTransactionMode::Readwrite,
                None => IdbTransactionMode::Readonly,
            },
            &vec!["configuration"],
        )?;

        let os = trans.object_store("configuration")?;

        if let Some(fct) = &self.fct {
            os.put_with_key(
                &serde_wasm_bindgen::to_value(fct)?,
                &serde_wasm_bindgen::to_value("fetcher")?,
            )?;
        }

        Ok(vec![os.get(&serde_wasm_bindgen::to_value("fetcher")?)?])
    }

    fn set_response(
        &mut self,
        result: Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>,
    ) -> Result<Option<repo::Response>, JsError> {
        match result {
            Ok(val) => Ok(Some(match val.is_undefined() {
                true => repo::Response::FetcherConfig(None),
                false => repo::Response::FetcherConfig(Some(serde_wasm_bindgen::from_value(val)?)),
            })),
            Err(_e) => Err("error getting fetcher config result".into()),
        }
    }
}
