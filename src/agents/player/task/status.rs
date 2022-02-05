use crate::{
    agents::{player::Response, repo},
    objects::JsError,
};

/// # Status Task
///
/// This task will return the status of the player (i.e., item, channel, length, playing).
/// If no source is set, ``None`` will be returned.
///
/// ## Stages and Transitions
///
/// * Finalize (S)
#[derive(Debug)]
pub struct StatusTask {
    stage: StatusStage,
}

#[derive(Debug)]
pub enum StatusStage {
    Finalize,
}

impl StatusTask {
    pub fn new() -> Self {
        Self {
            stage: StatusStage::Finalize,
        }
    }

    pub fn get_stage(&self) -> &StatusStage {
        &self.stage
    }
}

impl super::TaskProcessor<StatusTask> for super::super::Player {
    fn process(&mut self, task: &mut StatusTask) -> Result<bool, JsError> {
        match task.get_stage() {
            StatusStage::Finalize => {
                match &self.source {
                    Some(source) => {
                        self.send_response(Response::Status(Some((
                            source.0.clone(),
                            source.1.clone(),
                            self.audio_element.duration(),
                            !self.audio_element.paused(),
                        ))));
                    }
                    None => self.send_response(Response::Status(None)),
                }

                Ok(true)
            }
        }
    }
}
