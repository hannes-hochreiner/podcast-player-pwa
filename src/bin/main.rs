use podcast_player_pwa::components::Top;
#[allow(unused_imports)]
use wasm_bindgen::prelude::*;
#[allow(unused_imports)]
use yew::prelude::*;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<Top>::new().render();
}
