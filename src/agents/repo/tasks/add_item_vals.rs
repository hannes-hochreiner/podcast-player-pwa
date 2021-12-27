use crate::agents::repo;
use crate::objects::{Channel, Item, ItemVal, JsError};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use web_sys::{IdbDatabase, IdbTransaction, IdbTransactionMode};

pub struct AddItemValsTask {
    item_vals: Vec<ItemVal>,
    channels: Option<Vec<Channel>>,
    transaction: Option<IdbTransaction>,
}

impl AddItemValsTask {
    pub fn new_with_item_vals(item_vals: Vec<ItemVal>) -> Self {
        Self {
            item_vals,
            transaction: None,
            channels: None,
        }
    }
}

impl repo::RepositoryTask for AddItemValsTask {
    fn get_request(&mut self, db: &IdbDatabase) -> Result<Vec<web_sys::IdbRequest>, JsError> {
        if self.transaction.is_none() {
            self.transaction = Some(db.transaction_with_str_sequence_and_mode(
                &serde_wasm_bindgen::to_value(&vec!["items", "channels"])?,
                IdbTransactionMode::Readwrite,
            )?);
        }

        match (&self.transaction, &self.channels) {
            (Some(trans), None) => {
                let channels_os = trans.object_store("channels")?;
                let items_os = trans.object_store("items")?;

                Ok(vec![channels_os.get_all()?, items_os.get_all()?])
            }
            _ => Err("error adding item vals".into()),
        }
    }

    fn set_response(
        &mut self,
        result: Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>,
    ) -> Result<Option<repo::Response>, JsError> {
        match (&self.transaction, &mut self.channels) {
            (Some(_trans), None) => {
                self.channels = Some(serde_wasm_bindgen::from_value(result?)?);
                Ok(None)
            }
            (Some(trans), Some(channels)) => {
                let mut updated_channels = HashSet::<Uuid>::new();
                let mut channels: HashMap<Uuid, &mut Channel> =
                    channels.iter_mut().map(|e| (e.val.id, e)).collect();
                let items: Vec<Item> = serde_wasm_bindgen::from_value(result?)?;

                let item_os = trans.object_store("items")?;
                let item_map: HashMap<Uuid, &Item> =
                    items.iter().map(|e| (e.get_id(), e)).collect();

                for item in &self.item_vals {
                    let item_new = match item_map.get(&item.id) {
                        Some(&i) => {
                            let mut tmp_item = i.clone();

                            tmp_item.set_val(item);
                            channels
                                .get_mut(&item.channel_id)
                                .ok_or("error getting channel")?
                                .keys
                                .year_month_keys
                                .insert(tmp_item.get_year_month_key());
                            updated_channels.insert(item.channel_id);
                            tmp_item
                        }
                        None => item.into(),
                    };
                    item_os.put_with_key(
                        &serde_wasm_bindgen::to_value(&item_new)?,
                        &serde_wasm_bindgen::to_value(&item_new.get_id())?,
                    )?;
                }
                let channel_os = trans.object_store("channels")?;

                for channel_id in updated_channels {
                    let channel = channels.get(&channel_id).ok_or("error getting channel")?;
                    channel_os.put_with_key(
                        &serde_wasm_bindgen::to_value(*channel)?,
                        &serde_wasm_bindgen::to_value(&channel_id)?,
                    )?;
                }
                Ok(Some(repo::Response::AddItemVals(Ok(()))))
            }

            (None, _) => Err("error adding items vals".into()),
        }
    }
}
