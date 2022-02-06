mod agents;
mod components;
mod objects;
mod pages;
mod utils;

use components::Top;
#[allow(unused_imports)]
use wasm_bindgen::prelude::*;
#[allow(unused_imports)]
use yew::prelude::*;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<Top>();
}
