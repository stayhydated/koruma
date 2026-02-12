extern crate koruma_collection;
pub mod tui;

#[cfg(feature = "web")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    tui::run().unwrap();
}
