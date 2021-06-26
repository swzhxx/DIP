use crate::core::{conv::Convolution, kernel};
use js_sys::Uint8ClampedArray;
use ndarray::prelude::*;
use ndarray::Array;
use num_traits::ToPrimitive;
use wasm_bindgen::JsStatic;
use wasm_bindgen::{prelude::*, Clamped};
use wasm_bindgen_test::*;
use web_sys::{console, ImageData};

#[wasm_bindgen(js_name = splatGaussianFilter)]
pub fn splat_gaussian_filter(
    image_data: Uint8ClampedArray,
    width: usize,
    height: usize,
    kernel_size: usize,
) -> Result<ImageData, JsValue> {
    let gaussian_kernel = kernel::gaussian_kernel(kernel_size);

    let data: Vec<f64> = image_data
        .to_vec()
        .iter()
        .map(|val| (*val) as f64)
        .collect();

    let shape = (width, height, 4 as usize);
    let data = Array::from_shape_vec(shape, data).unwrap();
    // unsafe {
    //     web_sys::console::log_1(&format!("{:?}", shape).into());
    //     web_sys::console::log_1(&format!("{:?}", data).into());
    // }

    let filterd_data: Array3<f64> = data.conv_2d(&gaussian_kernel).unwrap();

    // .unwrap_or(Array::from_elem(shape, 255.))
    let filterd_data: Vec<u8> = filterd_data
        .into_raw_vec()
        .iter()
        .map(|val| *val as u8)
        .collect();

    ImageData::new_with_u8_clamped_array_and_sh(Clamped(&filterd_data), width as u32, height as u32)
}
