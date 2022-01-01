pub mod feed_val;

use feed_val::FeedVal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Feed {
    pub val: FeedVal,
}
