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

#[wasm_bindgen(js_name = takeNumberSliceBySharedRef)]
pub fn take_number_slice_by_shared_ref(slices: &mut [u8]) {
    for i in slices {
        unsafe { *i = 255 }
    }
}
