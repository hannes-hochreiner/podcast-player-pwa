use crate::agents::repo;
use crate::objects::{Channel, ChannelKeys, ChannelMeta, ChannelVal, JsError};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use web_sys::{IdbDatabase, IdbTransaction, IdbTransactionMode};

pub struct AddChannelValsTask {
    channel_vals: Vec<ChannelVal>,
    transaction: Option<IdbTransaction>,
}

impl AddChannelValsTask {
    pub fn new_with_channel_vals(channel_vals: Vec<ChannelVal>) -> Self {
        Self {
            channel_vals,
            transaction: None,
        }
    }
}

impl repo::RepositoryTask for AddChannelValsTask {
    fn get_request(&mut self, db: &IdbDatabase) -> Result<Vec<web_sys::IdbRequest>, JsError> {
        let trans = db
            .transaction_with_str_sequence_and_mode(
                &serde_wasm_bindgen::to_value(&vec!["channels"])?,
                IdbTransactionMode::Readwrite,
            )
            .unwrap();

        let os = trans.object_store("channels").unwrap();
        self.transaction = Some(trans);

        Ok(vec![os.get_all().unwrap()])
    }

    fn set_response(
        &mut self,
        result: Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>,
    ) -> Result<Option<repo::Response>, JsError> {
        match &self.transaction {
            Some(trans) => {
                let channels: Vec<Channel> = serde_wasm_bindgen::from_value(result?)?;

                let channel_os = trans.object_store("channels").unwrap();
                let channel_map: HashMap<Uuid, &Channel> =
                    channels.iter().map(|e| (e.val.id, e)).collect();

                for channel in &self.channel_vals {
                    let channel_new = match channel_map.get(&channel.id) {
                        Some(&c) => Channel {
                            val: channel.clone(),
                            meta: c.meta.clone(),
                            keys: c.keys.clone(),
                        },
                        None => {
                            let channel_id = channel.id;

                            Channel {
                                val: channel.clone(),
                                meta: ChannelMeta {
                                    id: channel_id,
                                    active: false,
                                },
                                keys: ChannelKeys {
                                    id: channel_id,
                                    year_month_keys: HashSet::new(),
                                },
                            }
                        }
                    };
                    channel_os
                        .put_with_key(
                            &serde_wasm_bindgen::to_value(&channel_new).unwrap(),
                            &serde_wasm_bindgen::to_value(&channel_new.val.id).unwrap(),
                        )
                        .unwrap();
                }
                Ok(Some(repo::Response::AddChannelVals(Ok(()))))
            }

            None => Err("error adding channel vals".into()),
        }
    }
}
