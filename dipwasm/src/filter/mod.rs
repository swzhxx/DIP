use crate::color::normalize_color;
use crate::core::{conv::Convolution, kernel};
use js_sys::Uint8ClampedArray;
use ndarray::prelude::*;
use ndarray::Array;
use ndarray::Zip;
use num_traits::Pow;
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

#[wasm_bindgen(js_name = splatColorEdge)]
pub fn splat_color_edge(
    image_data: Uint8ClampedArray,
    width: usize,
    height: usize,
) -> Result<ImageData, JsValue> {
    let sobel_y = kernel::Kernel::new(array![[1., 2., 1.], [0., 0., 0.], [-1., -2., -1.]]);
    let sobel_x = kernel::Kernel::new(array![[-1., 0., 1.], [-2., 0., 2.], [-1., 0., 1.]]);
    let image_data: Vec<f64> = image_data.to_vec().iter().map(|val| *val as f64).collect();
    let image_data = Array::from_shape_vec((width, height, 4), image_data).unwrap();
    let conv_y = image_data.conv_2d(&sobel_y).unwrap();
    let conv_x = image_data.conv_2d(&sobel_x).unwrap();
    unsafe {
        web_sys::console::log_1(&format!("{:?}", conv_y).into());
    }

    let mut theta = Array::from_elem((width, height), 0.);
    let mut f = Array::from_elem((width, height), 0.);
    {
        let square = |val: &f64| val.powi(2);
        let gxx = &conv_x.slice(s![.., .., 0]).map(square)
            + &conv_x.slice(s![.., .., 1]).map(square)
            + &conv_x.slice(s![.., .., 2]).map(square);

        let gyy = &conv_y.slice(s![.., .., 0]).map(square)
            + &conv_y.slice(s![.., .., 1]).map(square)
            + &conv_y.slice(s![.., .., 2]).map(square);

        let gxy = &conv_x.slice(s![.., .., 0]) * &conv_y.slice(s![.., .., 0])
            + &conv_x.slice(s![.., .., 1]) * &conv_y.slice(s![.., .., 1])
            + &conv_x.slice(s![.., .., 2]) * &conv_y.slice(s![.., .., 2]);
        Zip::from(&mut theta)
            .and(&gxx)
            .and(&gyy)
            .and(&gxy)
            .for_each(|w, &xx, &yy, &xy| {
                *w = 1. / 2. * (2. * xy / (xx - yy + 0.00000000001)).atan();
            });
        unsafe {
            web_sys::console::log_1(&format!("theta , {:?}", theta.shape()).into());
            web_sys::console::log_1(&format!("gxx , {:?}", gxx.shape()).into());
            web_sys::console::log_1(&format!("gyy , {:?}", gyy.shape()).into());
            web_sys::console::log_1(&format!("gxy , {:?}", gxy.shape()).into());
        }
        Zip::from(&mut f)
            .and(&gxx)
            .and(&gyy)
            .and(&gxy)
            .and(&theta)
            .for_each(|f, &xx, &yy, &xy, &theta| {
                *f = (1. / 2.
                    * ((xx + yy) + (xx - yy) * (2. * theta).cos() + 2. * xy * (2. * theta).sin()))
                .pow(1. / 2.)
            });

        unsafe {
            web_sys::console::log_1(&format!("f shape{:?}", f.shape()).into());
        }
    }

    // 填充 f的维度
    let extra = f
        .slice(s![.., ..])
        .into_shape((width, height, 1))
        .unwrap()
        .map(|val| *val);
    let extra = normalize_color(&extra);
    unsafe {
        web_sys::console::log_1(&format!("extra").into());
    }
    let mut f = normalize_color(&f.insert_axis(Axis(2)));
    unsafe {
        web_sys::console::log_1(&format!("f {:?} ", f).into());
    }
    f.append(Axis(2), extra.slice(s![.., .., ..])).unwrap();
    f.append(Axis(2), extra.slice(s![.., .., ..])).unwrap();
    f.append(
        Axis(2),
        Array::from_elem((width, height, 1), 255).slice(s![.., .., ..]),
    )
    .unwrap();

    let image_data: Vec<u8> = f.iter().map(|val| (*val)).collect();

    unsafe { web_sys::console::log_1(&format!("{:?}", image_data).into()) }
    ImageData::new_with_u8_clamped_array_and_sh(Clamped(&image_data), width as u32, height as u32)
}
