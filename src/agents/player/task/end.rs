use crate::{
    agents::{player::Response, repo},
    objects::JsError,
};

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

impl super::TaskProcessor<EndTask> for super::super::Player {
    fn process(&mut self, task: &mut EndTask) -> Result<bool, JsError> {
        match task.get_stage() {
            EndStage::Finalize => {
                if let Some(item) = &mut self.source_item {
                    item.increment_play_count();
                    item.set_current_time(None);
                    self.repo.send(repo::Request::UpdateItem(item.clone()));
                    self.send_response(Response::End);
                }

                Ok(true)
            }
        }
    }
}
