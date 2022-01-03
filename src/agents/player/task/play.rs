use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

use crate::agents::player::{Message, Response};

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

impl super::TaskProcessor<PlayTask> for super::super::Player {
    fn process(&mut self, task: &mut PlayTask) -> Result<bool, crate::objects::JsError> {
        match task.get_stage() {
            PlayStage::Init => {
                self.audio_element.add_event_listener_with_callback(
                    "timeupdate",
                    self.on_timeupdate_closure.as_ref().unchecked_ref(),
                )?;

                let prom = self.audio_element.play()?;

                self.link.send_future(async move {
                    Message::StartedPlaying(JsFuture::from(prom).await)
                });
                task.play_triggered();
                Ok(false)
            }
            PlayStage::WaitingForPlay => Ok(false),
            PlayStage::Finalize => {
                self.send_response(Response::Playing);
                Ok(true)
            }
        }
    }
}
