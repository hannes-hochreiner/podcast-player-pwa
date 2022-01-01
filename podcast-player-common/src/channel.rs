pub mod channel_keys;
pub mod channel_meta;
pub mod channel_val;

use channel_keys::ChannelKeys;
use channel_meta::ChannelMeta;
use channel_val::ChannelVal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Channel {
    pub val: ChannelVal,
    pub meta: ChannelMeta,
    pub keys: ChannelKeys,
}
