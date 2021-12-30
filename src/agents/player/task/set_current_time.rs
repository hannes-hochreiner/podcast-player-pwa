/// # Task to set the current time
///
/// ## Stages and Transitions
/// * Init (S)
/// * time_set (T)
/// * Finalize (S)
#[derive(Debug)]
pub enum SetCurrentTimeStage {
    Init,
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

    pub fn get_stage(&self) -> &SetCurrentTimeStage {
        &self.stage
    }

    pub fn get_current_time(&self) -> &f64 {
        &self.time
    }
}
