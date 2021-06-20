use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;
use web_sys::ImageData;
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    pub fn alert(s: &str);
    #[wasm_bindgen(js_namespace=console)]
    pub fn log(s: &str);
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

#[wasm_bindgen(js_name = makeImageData)]
pub fn make_image_data(data: ImageData) -> Result<ImageData, JsValue> {
    unsafe {
        log(&format!("{}", data.width()));
        log(&format!("{}", data.height()));
        let width = data.width();
        let height = data.height();
        let mut v: Vec<u8> = vec![];
        for i in 0..width * height * 4 {
            v.push(117);
        }
        ImageData::new_with_u8_clamped_array_and_sh(Clamped(&mut v), width, height)
    }
}
