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
use crate::objects::Item;
use js_sys::ArrayBuffer;

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

    pub fn get_stage(&self) -> &SetSourceStage {
        &self.stage
    }

    pub fn source_open_data_triggered(&mut self) {
        self.stage = SetSourceStage::WaitingForSourceOpenData;
    }

    pub fn source_opened(&mut self) {
        self.source_open = true;

        match (self.source_open, &self.data) {
            (true, Some(_)) => self.stage = SetSourceStage::SourceOpenData,
            (_, _) => self.stage = SetSourceStage::WaitingForSourceOpenData,
        }
    }

    pub fn data_buffer_update_triggered(&mut self) {
        self.stage = SetSourceStage::WaitingForBufferUpdate;
    }

    pub fn set_data(&mut self, data: ArrayBuffer) {
        self.data = Some(data);

        match (self.source_open, &self.data) {
            (true, Some(_)) => self.stage = SetSourceStage::SourceOpenData,
            (_, _) => self.stage = SetSourceStage::WaitingForSourceOpenData,
        }
    }

    pub fn get_data_ref(&self) -> Option<&ArrayBuffer> {
        self.data.as_ref()
    }

    pub fn get_item_ref(&self) -> &Item {
        &self.item
    }

    pub fn buffer_updated(&mut self) {
        self.stage = SetSourceStage::Finalize
    }
}
