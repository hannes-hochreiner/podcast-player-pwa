/// # Play Task
///
/// ## Stages and Transitions
///
/// * Init (S)
/// * play_triggered (T)
/// * WaitingForPlay (S)
/// * playing (T)
/// * Finalize (S)
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
