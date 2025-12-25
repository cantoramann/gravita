mod app;

#[cfg(target_arch = "wasm32")]
mod wasm {
    use wasm_bindgen::prelude::*;

    use super::app;

    #[wasm_bindgen(start)]
    pub fn wasm_start() -> Result<(), JsValue> {
        app::run_native().map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

pub use app::run_native;
