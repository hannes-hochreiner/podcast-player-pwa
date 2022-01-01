use crate::{
    agents::repo,
    objects::{Channel, JsError},
};
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
    fn get_request(&mut self, db: &IdbDatabase) -> Result<Vec<web_sys::IdbRequest>, JsError> {
        let transaction = db.transaction_with_str_sequence_and_mode(
            &serde_wasm_bindgen::to_value(&vec!["channels"])?,
            IdbTransactionMode::Readwrite,
        )?;
        let channel_os = transaction.object_store("channels")?;
        channel_os.put_with_key(
            &serde_wasm_bindgen::to_value(&self.channel)?,
            &serde_wasm_bindgen::to_value(&self.channel.val.id)?,
        )?;
        Ok(vec![channel_os.get_all()?])
    }

    fn set_response(
        &mut self,
        result: Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>,
    ) -> Result<Option<repo::Response>, JsError> {
        Ok(Some(repo::Response::Channels(
            serde_wasm_bindgen::from_value(result?)?,
        )))
    }
}
