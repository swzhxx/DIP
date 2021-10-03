use crate::utils::image_data_to_gray;
use ndarray::Array2;
use slam_core::features::fast::OFast;
use slam_core::matches::orb::{self, Orb};
use wasm_bindgen::prelude::*;
use web_sys::ImageData;

/// 特征点匹配
#[wasm_bindgen]
pub fn feature_point_matching(
    image_1: ImageData,
    image_2: ImageData,
    threshold: usize,
) -> Vec<usize> {
    // let image_data_1 = Array::from_shape_vec(())
    let gray_image_data_1: Array2<f64> = Array2::from_shape_vec(
        (image_1.height() as usize, image_1.width() as usize),
        image_data_to_gray(&image_1)
            .into_iter()
            .map(|v| v as f64)
            .collect(),
    )
    .unwrap();

    let gray_image_data_2: Array2<f64> = Array2::from_shape_vec(
        (image_2.height() as usize, image_2.width() as usize),
        image_data_to_gray(&image_1)
            .into_iter()
            .map(|v| v as f64)
            .collect(),
    )
    .unwrap();

    let ofast_1 = OFast::new(&gray_image_data_1);
    let ofast_2 = OFast::new(&gray_image_data_2);

    let features_1 = ofast_1.find_features();
    let features_2 = ofast_2.find_features();

    let descriptors_1 = Orb::new(&gray_image_data_1, &features_1).create_descriptors();
    let descriptors_2 = Orb::new(&gray_image_data_2, &features_2).create_descriptors();
    let matched = Orb::brief_match(
        &gray_image_data_1,
        &descriptors_1,
        &gray_image_data_2,
        &descriptors_2,
        threshold,
    );

    let match_points: Vec<usize> = matched.iter().fold(vec![], |mut acc, ele| {
        acc.push(ele.i1);
        acc.push(ele.i2);
        acc
    });
    match_points
}
