mod agents;
mod components;
mod objects;
mod pages;

use components::router::Router;
#[allow(unused_imports)]
use wasm_bindgen::prelude::*;
#[allow(unused_imports)]
use yew::prelude::*;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<Router>();
}
