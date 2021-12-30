use crate::objects::Item;
use js_sys::ArrayBuffer;

#[derive(Debug)]
pub struct PauseTask {
    stage: PauseStage,
}

#[derive(Debug)]
pub enum PauseStage {
    Init,
    WaitingForPause,
    Finalize,
}

impl PauseTask {
    pub fn new() -> Self {
        Self {
            stage: PauseStage::Init,
        }
    }

    pub fn pause_triggered(&mut self) {
        self.stage = PauseStage::WaitingForPause
    }

    pub fn paused(&mut self) {
        self.stage = PauseStage::Finalize
    }

    pub fn get_stage(&self) -> &PauseStage {
        &self.stage
    }
}

#[derive(Debug)]
pub struct PlayTask {
    stage: PlayStage,
}

#[derive(Debug)]
pub enum PlayStage {
    Init,
    WaitingForPlay,
    Finalize,
}

impl PlayTask {
    pub fn new() -> Self {
        Self {
            stage: PlayStage::Init,
        }
    }

    pub fn play_triggered(&mut self) {
        self.stage = PlayStage::WaitingForPlay;
    }

    pub fn playing(&mut self) {
        self.stage = PlayStage::Finalize
    }

    pub fn get_stage(&self) -> &PlayStage {
        &self.stage
    }
}

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

/// # Task to set the current time
///
/// ## Stages and Transitions
/// * Init (S)
/// * time_set (T)
/// * TimeSet (S)
/// * item_update_triggered (T)
/// * WaitingForItemUpdate (S)
/// * item_updated (T)
/// * Done (S)
#[derive(Debug)]
pub enum SetCurrentTimeStage {
    Init,
    // TimeSet,
    // WaitingForItemUpdate,
    // Done,
    Finalize,
}

#[derive(Debug)]
pub struct SetCurrentTimeTask {
    stage: SetCurrentTimeStage,
    time: f64,
}

impl SetCurrentTimeTask {
    pub fn new(time: f64) -> Self {
        Self {
            stage: SetCurrentTimeStage::Init,
            time,
        }
    }

    pub fn time_set(&mut self) {
        self.stage = SetCurrentTimeStage::Finalize;
    }

    // pub fn item_update_triggered(&mut self) {
    //     self.stage = SetCurrentTimeStage::WaitingForItemUpdate;
    // }

    // pub fn item_updated(&mut self) {
    //     self.stage = SetCurrentTimeStage::Done;
    // }

    pub fn get_stage(&self) -> &SetCurrentTimeStage {
        &self.stage
    }

    pub fn get_current_time(&self) -> &f64 {
        &self.time
    }
}
