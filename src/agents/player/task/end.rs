/// # End Task
///
/// It is assumed that this task will only be created in the on_end event handler.
///
/// ## Stages and Transitions
///
/// * Finalize (S)
#[derive(Debug)]
pub struct EndTask {
    stage: EndStage,
}

#[derive(Debug)]
pub enum EndStage {
    Finalize,
}

impl EndTask {
    pub fn new() -> Self {
        Self {
            stage: EndStage::Finalize,
        }
    }

    pub fn get_stage(&self) -> &EndStage {
        &self.stage
    }
}
