pub mod item_keys;
pub mod item_meta;
pub mod item_val;

use chrono::{DateTime, FixedOffset};
use item_keys::ItemKeys;
use item_meta::{DownloadStatus, ItemMeta};
use item_val::ItemVal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Item {
    val: ItemVal,
    meta: ItemMeta,
    keys: ItemKeys,
}

impl Item {
    pub fn get_id(&self) -> Uuid {
        self.val.id
    }

    pub fn get_title(&self) -> String {
        self.val.title.clone()
    }

    pub fn get_date(&self) -> DateTime<FixedOffset> {
        self.val.date
    }

    pub fn get_new(&self) -> bool {
        self.meta.new
    }

    pub fn set_new(&mut self, new: bool) {
        self.meta.new = new;
    }

    pub fn set_val(&mut self, val: &ItemVal) {
        self.val = val.clone();
        self.regenerate_keys();
    }

    pub fn get_download(&self) -> bool {
        self.meta.download
    }

    pub fn set_download(&mut self, download: bool) {
        self.meta.download = download;
        self.meta.new = false;

        match (&self.meta.download, &self.meta.download_status) {
            (true, DownloadStatus::NotRequested) => {
                self.meta.download_status = DownloadStatus::Pending
            }
            (false, DownloadStatus::Pending) => {
                self.meta.download_status = DownloadStatus::NotRequested
            }
            (_, _) => {}
        }
        self.regenerate_keys();
    }

    pub fn get_download_status(&self) -> DownloadStatus {
        self.meta.download_status.clone()
    }

    pub fn set_download_status(&mut self, download_status: DownloadStatus) {
        self.meta.download_status = download_status;
        self.regenerate_keys();
    }

    pub fn get_year_month_key(&self) -> String {
        self.keys.year_month.clone()
    }

    pub fn set_current_time(&mut self, time: Option<f64>) {
        self.meta.current_time = time;
    }

    pub fn get_current_time(&self) -> Option<f64> {
        self.meta.current_time.clone()
    }

    fn regenerate_keys(&mut self) {
        self.keys = ItemKeys::new_from_val_meta(&self.val, &self.meta);
    }

    pub fn increment_play_count(&mut self) {
        self.meta.play_count = self.meta.play_count + 1;
    }

    pub fn get_play_count(&self) -> u32 {
        self.meta.play_count
    }
}

impl From<&ItemVal> for Item {
    fn from(val: &ItemVal) -> Self {
        let meta = ItemMeta {
            id: val.id,
            new: true,
            download: false,
            download_status: DownloadStatus::NotRequested,
            current_time: None,
            play_count: 0,
        };
        let keys = ItemKeys::new_from_val_meta(&val, &meta);

        Self {
            val: val.clone(),
            meta,
            keys,
        }
    }
}
