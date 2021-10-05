use ndarray::{s, Array2, Array3};
use wasm_bindgen::prelude::*;
use web_sys::ImageData;
pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

pub fn rgba_to_gray(img: &Array3<u8>) -> Array2<u8> {
    let shape = img.shape();
    let height = shape[0];
    let width = shape[1];

    let mut color_data = vec![];
    for y in 0..height {
        for x in 0..width {
            let rgba = img.slice(s![y, x, ..]);
            let gray = (rgba[0] * 30 + rgba[1] * 59 + rgba[2] * 11 + 50) / 100;
            color_data.push(gray)
        }
    }
    let gray_data = Array2::from_shape_vec((height, width), color_data).unwrap();
    // web_sys::console::log_1(&format!("gray data {:?}", gray_data).into());
    gray_data
}

pub fn image_data_to_gray(img: &ImageData) -> Array2<u8> {
    let width = img.width();
    let height = img.height();
    let array_data = Array3::from_shape_vec(
        (height as usize, width as usize, 4usize),
        img.data().to_vec(),
    )
    .unwrap();

    let gray_image_data = rgba_to_gray(&array_data);
    gray_image_data
}
