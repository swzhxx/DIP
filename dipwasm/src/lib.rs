use js_sys::Math;
use ndarray::prelude::*;
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

#[wasm_bindgen(js_name=letsPapperNoise)]
pub fn lets_papper_noise(data: ImageData) -> Result<ImageData, JsValue> {
    let width = data.width() as usize;
    let height = data.height() as usize;
    let data: &Vec<u8> = &data.data();
    let mut a: Array3<u8> = Array::from_shape_vec((width, height, 4), data.to_vec()).unwrap();
    for y in 0..height {
        for x in 0..width {
            let r = unsafe { Math::random() };
            if r < 0.99 {
                continue;
            }
            if r > 0.995 {
                a.slice_mut(s![y, x, ..]).fill(255 as u8);
            } else {
                a.slice_mut(s![y, x, ..])
                    .assign(&array![0 as u8, 0 as u8, 0 as u8, 255 as u8].slice(s![..]));
            }
        }
    }

    //
    let mut data = Clamped(a.into_raw_vec());
    ImageData::new_with_u8_clamped_array_and_sh(Clamped(&mut data), width as u32, height as u32)
}

pub mod color;

pub mod filter;

pub mod core;
