use crate::agents::repo;
use crate::objects::Channel;
use anyhow::{anyhow, Result};
use web_sys::{IdbDatabase, IdbTransactionMode};

pub struct UpdateChannelTask {
    channel: Channel,
}

impl UpdateChannelTask {
    pub fn new_with_channel(channel: Channel) -> Self {
        Self { channel }
    }
}

impl repo::RepositoryTask for UpdateChannelTask {
    fn get_request(&mut self, db: &IdbDatabase) -> anyhow::Result<Vec<web_sys::IdbRequest>> {
        let transaction = db
            .transaction_with_str_sequence_and_mode(
                &serde_wasm_bindgen::to_value(&vec!["channels"]).unwrap(),
                IdbTransactionMode::Readwrite,
            )
            .unwrap();
        let channel_os = transaction.object_store("channels").unwrap();
        channel_os
            .put_with_key(
                &serde_wasm_bindgen::to_value(&self.channel).unwrap(),
                &serde_wasm_bindgen::to_value(&self.channel.val.id).unwrap(),
            )
            .unwrap();
        Ok(vec![channel_os.get_all().unwrap()])
    }

    fn set_response(
        &mut self,
        result: Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>,
    ) -> anyhow::Result<Option<repo::Response>> {
        Ok(Some(repo::Response::Channels(
            serde_wasm_bindgen::from_value(
                result.map_err(|_e| anyhow!("error getting channel result"))?,
            )
            .map_err(|_e| anyhow!("error converting channel result"))?,
        )))
    }
}
