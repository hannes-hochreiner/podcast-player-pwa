mod end;
mod pause;
mod play;
mod set_current_time;
mod set_source;

pub use end::*;
pub use pause::*;
pub use play::*;
pub use set_current_time::*;
pub use set_source::*;

use crate::objects::JsError;

#[derive(Debug)]
pub enum Task {
    SetSource(SetSourceTask),
    Play(PlayTask),
    Pause(PauseTask),
    SetCurrentTime(SetCurrentTimeTask),
    End(EndTask),
}

pub trait TaskProcessor<T> {
    fn process(&mut self, task: &mut T) -> Result<bool, JsError>;
}
