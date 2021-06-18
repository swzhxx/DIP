use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    pub fn alert(s: &str);
}

#[wasm_bindgen]
pub fn wasmalert(s: &str) {
    unsafe {
        alert(&format!("From Rust2 {}!", s));
    }
}
