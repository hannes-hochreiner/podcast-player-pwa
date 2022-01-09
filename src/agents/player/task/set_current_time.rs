use crate::agents::repo;

#[derive(Debug)]
pub enum SetCurrentTimeStage {
    Init,
    Finalize,
}

/// # Task to set the current time
///
/// ## Stages and Transitions
/// * Init (S)
/// * time_set (T)
/// * Finalize (S)
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

impl super::TaskProcessor<SetCurrentTimeTask> for super::super::Player {
    fn process(&mut self, task: &mut SetCurrentTimeTask) -> Result<bool, crate::objects::JsError> {
        match task.get_stage() {
            SetCurrentTimeStage::Init => {
                self.audio_element
                    .set_current_time(task.get_current_time().clone());
                Ok(false)
            }
            SetCurrentTimeStage::Finalize => {
                if let Some(item) = &mut self.source_item {
                    item.set_current_time(Some(self.audio_element.current_time()));
                    self.repo.send(repo::Request::UpdateItem(item.clone()));
                }
                Ok(true)
            }
        }
    }
}
