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
