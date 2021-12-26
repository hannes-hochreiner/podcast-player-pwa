use crate::agents::repo;
use crate::objects::{Feed, FeedVal};
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use uuid::Uuid;
use web_sys::{IdbDatabase, IdbTransaction, IdbTransactionMode};

pub struct AddFeedValsTask {
    feed_vals: Vec<FeedVal>,
    transaction: Option<IdbTransaction>,
}

impl AddFeedValsTask {
    pub fn new_with_feed_vals(feed_vals: Vec<FeedVal>) -> Self {
        Self {
            feed_vals,
            transaction: None,
        }
    }
}

impl repo::RepositoryTask for AddFeedValsTask {
    fn get_request(&mut self, db: &IdbDatabase) -> anyhow::Result<Vec<web_sys::IdbRequest>> {
        let trans = db
            .transaction_with_str_sequence_and_mode(
                &serde_wasm_bindgen::to_value(&vec!["feeds"]).unwrap(),
                IdbTransactionMode::Readwrite,
            )
            .unwrap();

        let os = trans.object_store("feeds").unwrap();
        self.transaction = Some(trans);

        Ok(vec![os.get_all().unwrap()])
    }

    fn set_response(
        &mut self,
        result: Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>,
    ) -> anyhow::Result<Option<repo::Response>> {
        match &self.transaction {
            Some(trans) => {
                let feeds: Vec<Feed> = serde_wasm_bindgen::from_value(
                    result.map_err(|_e| anyhow!("error getting feed result"))?,
                )
                .map_err(|_e| anyhow!("error converting feed result"))?;

                let feed_os = trans.object_store("feeds").unwrap();
                let feed_map: HashMap<Uuid, &Feed> = feeds.iter().map(|e| (e.val.id, e)).collect();

                for feed in &self.feed_vals {
                    let feed_new = match feed_map.get(&feed.id) {
                        Some(&c) => Feed { val: feed.clone() },
                        None => {
                            let feed_id = feed.id;

                            Feed { val: feed.clone() }
                        }
                    };
                    feed_os
                        .put_with_key(
                            &serde_wasm_bindgen::to_value(&feed_new).unwrap(),
                            &serde_wasm_bindgen::to_value(&feed_new.val.id).unwrap(),
                        )
                        .unwrap();
                }
                Ok(Some(repo::Response::AddFeedVals(Ok(()))))
            }

            None => Err(anyhow!("error adding feed vals")),
        }
    }
}
