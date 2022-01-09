use crate::{
    agents::{player::Response, repo},
    objects::Item,
};
use js_sys::ArrayBuffer;
use wasm_bindgen::JsCast;
use web_sys::{MediaSource, Url};

use super::TaskProcessor;

/// # Set Source Task
///
/// ## Stages and Transitions
///
/// * Init (S)
/// * source_open_data_triggered (T)
/// * WaitingForSourceOpenData (S)
/// * source_opened && set_data (T)
/// * SourceOpenData (S)
/// * data_buffer_update_triggered (T)
/// * WaitingForBufferUpdate (S)
/// * buffer_updated (T)
/// * Finalize (S)
#[derive(Debug)]
pub struct SetSourceTask {
    item: Item,
    data: Option<ArrayBuffer>,
    source_open: bool,
    stage: SetSourceStage,
}

#[derive(Debug)]
pub enum SetSourceStage {
    Init,
    WaitingForSourceOpenData,
    SourceOpenData,
    WaitingForBufferUpdate,
    Finalize,
}

impl SetSourceTask {
    pub fn new(item: Item) -> Self {
        Self {
            item,
            data: None,
            source_open: false,
            stage: SetSourceStage::Init,
        }
    }

    pub fn source_opened(&mut self) {
        self.source_open = true;

        match (self.source_open, &self.data) {
            (true, Some(_)) => self.stage = SetSourceStage::SourceOpenData,
            (_, _) => self.stage = SetSourceStage::WaitingForSourceOpenData,
        }
    }

    pub fn set_data(&mut self, data: ArrayBuffer) {
        self.data = Some(data);

        match (self.source_open, &self.data) {
            (true, Some(_)) => self.stage = SetSourceStage::SourceOpenData,
            (_, _) => self.stage = SetSourceStage::WaitingForSourceOpenData,
        }
    }

    pub fn buffer_updated(&mut self) {
        self.stage = SetSourceStage::Finalize
    }
}

impl TaskProcessor<SetSourceTask> for super::super::Player {
    fn process(&mut self, task: &mut SetSourceTask) -> Result<bool, crate::objects::JsError> {
        match &task.stage {
            SetSourceStage::Init => {
                // remove source
                self.source = None;
                // set new media source
                self.media_source = MediaSource::new()?;
                self.audio_element
                    .set_src(&Url::create_object_url_with_source(&self.media_source)?);
                self.media_source.set_onsourceopen(Some(
                    self.mediasource_opened_closure.as_ref().unchecked_ref(),
                ));
                // request data
                self.repo
                    .send(repo::Request::GetEnclosure(task.item.get_id()));
                task.stage = SetSourceStage::WaitingForSourceOpenData;
                Ok(false)
            }
            SetSourceStage::WaitingForSourceOpenData => Ok(false),
            SetSourceStage::SourceOpenData => {
                // clear existing source buffers
                let sbl = self.media_source.source_buffers();

                while let Some(sb) = sbl.get(0) {
                    self.media_source.remove_source_buffer(&sb)?
                }
                // create new source buffer
                let sb = self.media_source.add_source_buffer("audio/mpeg")?;
                self.audio_element.set_preload("metadata");
                // load data
                sb.append_buffer_with_array_buffer(
                    task.data.as_ref().ok_or("could not get data")?,
                )?;
                sb.set_onupdate(Some(
                    self.sourcebuffer_update_closure.as_ref().unchecked_ref(),
                ));
                task.stage = SetSourceStage::WaitingForBufferUpdate;
                Ok(false)
            }
            SetSourceStage::WaitingForBufferUpdate => Ok(false),
            SetSourceStage::Finalize => {
                self.audio_element.set_playback_rate(1.5);
                self.audio_element.set_volume(1.0);
                self.audio_element
                    .set_current_time(match task.item.get_current_time() {
                        Some(current_time) => current_time,
                        None => 0.0,
                    });
                self.source = Some(task.item.clone());
                self.send_response(Response::SourceSet(
                    task.item.clone(),
                    self.audio_element.duration(),
                ));
                Ok(true)
            }
        }
    }
}
