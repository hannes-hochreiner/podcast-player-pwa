use crate::{agents::repo, objects::FetcherConfig};
use anyhow::{anyhow, Result};
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
    fn get_request(&mut self, db: &IdbDatabase) -> anyhow::Result<Vec<web_sys::IdbRequest>> {
        let trans = self.create_transaction(
            &db,
            match &self.fct {
                Some(_) => IdbTransactionMode::Readwrite,
                None => IdbTransactionMode::Readonly,
            },
            &vec!["configuration"],
        )?;

        let os = trans
            .object_store("configuration")
            .map_err(|_e| anyhow!("error creating object store"))?;

        if let Some(fct) = &self.fct {
            os.put_with_key(
                &serde_wasm_bindgen::to_value(fct).unwrap(),
                &serde_wasm_bindgen::to_value("fetcher").unwrap(),
            )
            .unwrap();
        }

        Ok(vec![os
            .get(
                &serde_wasm_bindgen::to_value("fetcher")
                    .map_err(|_e| anyhow!("error converting key"))?,
            )
            .map_err(|_e| {
                anyhow!("error items with channel id and year month")
            })?])
    }

    fn set_response(
        &mut self,
        result: Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>,
    ) -> anyhow::Result<Option<repo::Response>> {
        match result {
            Ok(val) => Ok(Some(match val.is_undefined() {
                true => repo::Response::FetcherConfig(None),
                false => repo::Response::FetcherConfig(Some(
                    serde_wasm_bindgen::from_value(val)
                        .map_err(|_e| anyhow!("error converting fetcher config result"))?,
                )),
            })),
            Err(_e) => Err(anyhow!("error getting fetcher config result")),
        }
    }
}
