use wasm_bindgen::JsCast;

use crate::agents::{player::Response, repo};

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

impl super::TaskProcessor<PauseTask> for super::super::Player {
    fn process(&mut self, task: &mut PauseTask) -> Result<bool, crate::objects::JsError> {
        match task.get_stage() {
            PauseStage::Init => {
                if self.audio_element.paused() {
                    Ok(true)
                } else {
                    self.audio_element.pause()?;
                    task.pause_triggered();
                    Ok(false)
                }
            }
            PauseStage::WaitingForPause => Ok(false),
            PauseStage::Finalize => {
                if let Some(curr_item) = &mut self.source_item {
                    self.audio_element.remove_event_listener_with_callback(
                        "timeupdate",
                        self.on_timeupdate_closure.as_ref().unchecked_ref(),
                    )?;
                    curr_item.set_playback_time(Some(self.audio_element.current_time()));
                    self.repo.send(repo::Request::UpdateItem(curr_item.clone()));
                    self.send_response(Response::Paused);
                }
                Ok(true)
            }
        }
    }
}
