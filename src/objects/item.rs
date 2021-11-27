use chrono::{DateTime, FixedOffset};
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

    fn regenerate_keys(&mut self) {
        self.keys = ItemKeys::new_from_val_meta(&self.val, &self.meta);
    }
}

impl From<&ItemVal> for Item {
    fn from(val: &ItemVal) -> Self {
        let meta = ItemMeta {
            id: val.id,
            new: true,
            download: false,
            download_status: DownloadStatus::NotRequested,
        };
        let keys = ItemKeys::new_from_val_meta(&val, &meta);

        Self {
            val: val.clone(),
            meta,
            keys,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ItemVal {
    pub id: Uuid,
    pub title: String,
    pub date: DateTime<FixedOffset>,
    pub enclosure_type: String,
    pub enclosure_url: String,
    pub channel_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ItemMeta {
    id: Uuid,
    new: bool,
    download: bool,
    download_status: DownloadStatus,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DownloadStatus {
    NotRequested,
    Pending,
    InProgress,
    Ok(u32),
    Error,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ItemKeys {
    pub id: Uuid,
    pub year_month: String,
    pub download_required: String,
    pub download_ok: String,
}

impl ItemKeys {
    pub fn new_from_val_meta(val: &ItemVal, meta: &ItemMeta) -> Self {
        let download_required = match (&meta.download, &meta.download_status) {
            (true, DownloadStatus::Pending) => String::from("true"),
            _ => String::from("false"),
        };
        let download_ok = match &meta.download_status {
            &DownloadStatus::Ok(_) => String::from("true"),
            _ => String::from("false"),
        };

        Self {
            id: val.id,
            year_month: val.date.to_rfc3339()[0..7].to_string(),
            download_required,
            download_ok,
        }
    }
}
