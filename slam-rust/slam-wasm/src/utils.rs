use ndarray::{s, Array, Array2, Array3};
use ndarray_stats::QuantileExt;
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
    // web_sys::console::log_1(&format!("img{:?}", img).into());
    let mut color_data = vec![];
    for y in 0..height {
        for x in 0..width {
            let rgba = img.slice(s![y, x, ..]);
            let gray =
                ((rgba[0] as usize) * 30 + (rgba[1] as usize) * 59 + (rgba[2] as usize) * 11 + 50)
                    / 100;
            let gray = gray as u8;
            // if y == 0 && x == 0 {
            //     web_sys::console::log_1(&format!("rgba {:?}", rgba).into());
            //     web_sys::console::log_1(&format!("gray {:?}", gray).into());
            // }
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

pub fn nomalize_gray_color(img: &Array2<f64>) -> Array2<f64> {
    let max = img.max().unwrap().clone();
    let min = img.min().unwrap().clone();
    // let mean = img.mean().unwrap();
    web_sys::console::log_1(&format!("max  {:?}", max).into());
    web_sys::console::log_1(&format!("min {:?}", min).into());
    let height = img.shape()[0];
    let width = img.shape()[1];
    let mut temp = Array::from_elem((height, width), 0.);
    for y in 0..height {
        for x in 0..width {
            let value = img[[y, x]];
            temp[[y, x]] = (value - min) / (max - min)
        }
    }

    temp = temp * 255.;
    return temp;
}
