use js_sys::Uint8ClampedArray;
use ndarray::{prelude::*, Zip};
use wasm_bindgen::{prelude::*, Clamped};
use web_sys::ImageData;

// ? 如何得到的这量化表
// 标准亮度量化表
fn qy() -> Array2<f64> {
    return array![
        [16., 11., 10., 16., 24., 40., 51., 61.],
        [12., 12., 14., 19., 26., 58., 60., 55.],
        [14., 13., 16., 24., 40., 57., 69., 56.],
        [14., 17., 22., 29., 51., 87., 80., 62.],
        [18., 22., 37., 56., 68., 109., 103., 77.],
        [24., 35., 55., 64., 81., 104., 113., 92.],
        [49., 64., 78., 87., 103., 121., 120., 101.],
        [72., 92., 95., 98., 112., 100., 103., 99.]
    ];
}

//标准色度量化表
fn qc() -> Array2<f64> {
    return array![
        [17., 18., 24., 47., 99., 99., 99., 99.],
        [18., 21., 26., 66., 99., 99., 99., 99.],
        [24., 26., 56., 99., 99., 99., 99., 99.],
        [47., 66., 99., 99., 99., 99., 99., 99.],
        [99., 99., 99., 99., 99., 99., 99., 99.],
        [99., 99., 99., 99., 99., 99., 99., 99.],
        [99., 99., 99., 99., 99., 99., 99., 99.],
        [99., 99., 99., 99., 99., 99., 99., 99.]
    ];
}

use crate::{
    color::{RGB, YUV},
    core::transform::{dct_2d_by_64_blocks, inverse_dct_2d_by_64_blocks},
};
#[wasm_bindgen(js_name=splatJpeg)]
pub fn splat_jpeg(
    data: Uint8ClampedArray,
    width: usize,
    height: usize,
) -> Result<ImageData, JsValue> {
    let mut QY = qy();
    let mut QC = qc();

    let times = width / 8;

    for time in 0..times * times - 1 {
        QY.append(Axis(0), qy().slice(s![.., ..]));

        QC.append(Axis(0), qc().slice(s![.., ..]));
    }
    unsafe {
        web_sys::console::log_1(&format!("{:?}", QY.shape()).into());
    }
    QY = QY.into_shape([width, height]).unwrap();
    QC = QC.into_shape([width, height]).unwrap();
    let image_data = Array::from_shape_vec(
        [width, height, 4],
        data.to_vec()
            .iter()
            .map(|val| {
                return *val as f64;
            })
            .collect(),
    )
    .unwrap();

    // remove alpha
    let mut image_data = image_data
        .slice(s![.., .., 0..3])
        .map(|val| *val)
        .into_shape([width, height, 3])
        .unwrap();
    unsafe {
        web_sys::console::log_1(&format!("1").into());
    }
    // color space to yuv
    for y in 0..height {
        for x in 0..width {
            let mut pixel_slice = image_data.slice_mut(s![y, x, ..]);
            let rgb = RGB::new(
                pixel_slice[0] as u8,
                pixel_slice[1] as u8,
                pixel_slice[2] as u8,
            );
            let yui = rgb.to_yuv();
            pixel_slice.assign(&array![yui.0, yui.1, yui.2].slice(s![..]));
        }
    }
    unsafe {
        web_sys::console::log_1(&format!("1.1").into());
    }
    // sampling , 2*2 blocks 4:1:1 sampling
    let mut y = 0;
    while y <= height - 2 {
        let mut x = 0;
        while x <= width - 2 {
            let mut block = image_data.slice_mut(s![y..y + 2, x..x + 2, ..]);
            let will_u = block[[0, 0, 1]];
            let will_v = block[[0, 0, 2]];

            block[[0, 1, 1]] = will_u;
            block[[0, 1, 2]] = will_v;
            block[[1, 0, 1]] = will_u;
            block[[1, 0, 2]] = will_v;
            block[[1, 1, 1]] = will_u;
            block[[1, 1, 2]] = will_v;
            x = x + 2
        }
        y = y + 2
    }
    unsafe {
        web_sys::console::log_1(&format!("1.2").into());
    }
    //dct transform
    let image_data_y = dct_2d_by_64_blocks(&image_data.slice(s![.., .., 0]).mapv(|v| v));
    let image_data_u = dct_2d_by_64_blocks(&image_data.slice(s![.., .., 1]).mapv(|v| v));
    let image_data_v = dct_2d_by_64_blocks(&image_data.slice(s![.., .., 2]).mapv(|v| v));

    // Quantitative data
    let by = (&image_data_y / &QY).mapv(|val| val);
    let bu = (&image_data_u / &QC).mapv(|val| val);
    let bv = (&image_data_v / &QC).mapv(|val| val);

    // inverse
    unsafe {
        web_sys::console::log_1(&format!("1.4").into());
    }
    // inverse Quantiative data
    let iby = (&by * &QY).mapv(|val| val as f64);
    let ibu = (&bu * &QC).mapv(|val| val as f64);
    let ibv = (&bv * &QC).mapv(|val| val as f64);
  
    unsafe {
        web_sys::console::log_1(&format!("{:?},{:?},{:?}", by, bu, bv).into());
    }
    //inverse dct
    let image_data_y = inverse_dct_2d_by_64_blocks(&iby);
    let image_data_u = inverse_dct_2d_by_64_blocks(&ibu);
    let image_data_v = inverse_dct_2d_by_64_blocks(&ibv);
    unsafe {
        web_sys::console::log_1(&format!("1.6").into());
    }
    //inverse imagedata
    let mut image_data = Array::from_elem([width, height, 4], 255.);
    image_data.slice_mut(s![.., .., 0]).assign(&image_data_y);
    image_data.slice_mut(s![.., .., 1]).assign(&image_data_u);
    image_data.slice_mut(s![.., .., 2]).assign(&image_data_v);

    //to rgb
    for y in 0..height {
        for x in 0..width {
            let mut pixel_slice = image_data.slice_mut(s![y, x, 0..3]);

            let yuv = YUV::new(pixel_slice[0], pixel_slice[1], pixel_slice[2]);

            let rgb = yuv.to_rgb();
            pixel_slice.assign(&array![rgb.0 as f64, rgb.1 as f64, rgb.2 as f64].slice(s![..]));
        }
    }
    unsafe {
        web_sys::console::log_1(&format!("2.1").into());
    }
    let mut image_data = image_data.mapv(|val| val as u8);
    unsafe {
        web_sys::console::log_1(&format!("2.5").into());
    }

    unsafe {
        web_sys::console::log_1(&format!("3").into());
    }
    let image_data: Vec<u8> = image_data.iter().map(|val| *val).collect();
    ImageData::new_with_u8_clamped_array_and_sh(Clamped(&image_data), width as u32, height as u32)
}

#[cfg(test)]
mod test {}
