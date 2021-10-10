use crate::utils::{image_data_to_gray, set_panic_hook};
use ndarray::Array2;
use slam_core::features::fast::OFast;
use slam_core::matches::orb::{self, BriefDescriptor, Orb};
use slam_core::matches::DMatch;
use slam_core::point::Point2;
use wasm_bindgen::prelude::*;
use web_sys::ImageData;

#[wasm_bindgen]

/// ORB 特征匹配，使用Brief算法
/// feature_points 坐标x,y的集合，长度为偶数
/// matched 为坐标x,y,d的集合，长度为3的倍数
struct OrbFeatureMatcher {
    image_1: ImageData,
    image_2: ImageData,
    feature_points_1: Vec<usize>,
    feature_points_2: Vec<usize>,
    matched: Vec<usize>,
}

#[wasm_bindgen]
impl OrbFeatureMatcher {
    #[wasm_bindgen(constructor)]
    pub fn new(image_1: ImageData, image_2: ImageData) -> Self {
        set_panic_hook();
        OrbFeatureMatcher {
            image_1,
            image_2,
            feature_points_1: vec![],
            feature_points_2: vec![],
            matched: vec![],
        }
    }

    pub fn get_feature_points_1(&self) -> Vec<usize> {
        self.feature_points_1.clone()
    }

    pub fn get_feature_points_2(&self) -> Vec<usize> {
        self.feature_points_2.clone()
    }

    pub fn get_matched(&self) -> Vec<usize> {
        self.matched.clone()
    }

    pub fn feature_point_matching(&mut self, threshold: u32, feature_threshold: Option<f64>) {
        let image_1 = &self.image_1;
        let image_2 = &self.image_2;
        let gray_image_data_1: Array2<f64> = Array2::from_shape_vec(
            (image_1.height() as usize, image_1.width() as usize),
            image_data_to_gray(image_1)
                .into_iter()
                .map(|v| v as f64)
                .collect(),
        )
        .unwrap();

        let gray_image_data_2: Array2<f64> = Array2::from_shape_vec(
            (image_2.height() as usize, image_2.width() as usize),
            image_data_to_gray(image_2)
                .into_iter()
                .map(|v| v as f64)
                .collect(),
        )
        .unwrap();

        let ofast_1 = OFast::new(&gray_image_data_1);
        let ofast_2 = OFast::new(&gray_image_data_2);
        // web_sys::console::log_1(&format!(" gray_image_data_1 {:?}", gray_image_data_1).into());

        let features_1 = ofast_1.find_features(feature_threshold);
        let features_2 = ofast_2.find_features(feature_threshold);
        self.feature_points_1 = features_1.iter().fold(vec![], |mut acc, p| {
            acc.push(p.x);
            acc.push(p.y);
            acc
        });
        self.feature_points_2 = features_2.iter().fold(vec![], |mut acc, p| {
            acc.push(p.x);
            acc.push(p.y);
            acc
        });
        // web_sys::console::log_1(&format!("feature_points_1 {:?}", &self.feature_points_1).into());
        // web_sys::console::log_1(&format!("create descriptors").into());
        let descriptors_1 = Orb::new(&gray_image_data_1, &features_1).create_descriptors();
        web_sys::console::log_1(&format!(" descriptors_1 {:?}", descriptors_1).into());
        let descriptors_2 = Orb::new(&gray_image_data_2, &features_2).create_descriptors();
        web_sys::console::log_1(&format!(" descriptors_2 {:?}", descriptors_2).into());
        let matched = Orb::brief_match(&descriptors_1, &descriptors_2, threshold);

        let match_points: Vec<usize> = matched.iter().fold(vec![], |mut acc, ele| {
            acc.push(ele.i1);
            acc.push(ele.i2);
            acc.push(ele.distance as usize);
            acc
        });
        self.matched = match_points;
        // match_points
    }
}
