use anyhow::{anyhow, Result};
use opencv::core::{DMatch, KeyPoint, Vector};
use opencv::{self as cv, prelude::*};
pub trait FeatureProcess {
    type Output;
    fn extract_features(&mut self) -> Self::Output;
}

pub struct SiftFeatureProcess {
    img: cv::core::Mat,
    key_points: Vector<KeyPoint>,
    desc: cv::core::Mat,
}

impl SiftFeatureProcess {
    pub fn new(img: cv::core::Mat) -> Self {
        Self {
            img,
            key_points: Default::default(),
            desc: Default::default(),
        }
    }

    pub fn get_desc(&self) -> &cv::core::Mat {
        &self.desc
    }
    pub fn get_key_points(&self) -> &Vector<KeyPoint> {
        &self.key_points
    }
}

impl Default for SiftFeatureProcess {
    fn default() -> Self {
        Self {
            key_points: Default::default(),
            desc: cv::core::Mat::default(),
            img: cv::core::Mat::default(),
        }
    }
}

impl FeatureProcess for SiftFeatureProcess {
    type Output = Result<()>;
    fn extract_features(&mut self) -> Self::Output {
        let mut sift = cv::features2d::SIFT::create(0, 3, 0.04, 10., 1.6)?;
        let mask = cv::core::Mat::default();
        sift.detect_and_compute(
            &self.img,
            &mask,
            &mut self.key_points,
            &mut self.desc,
            false,
        )?;
        Ok(())
    }
}

/// 处理特征点和Desc。获取
pub struct FeaturePointMatchBuilder<'a> {
    matches: Vec<DMatch>,
    desc1: &'a cv::core::Mat,
    desc2: &'a cv::core::Mat,
    key_points_1: &'a Vector<KeyPoint>,
    key_points_2: &'a Vector<KeyPoint>,
}

impl<'a> FeaturePointMatchBuilder<'a> {
    pub fn new(
        desc1: &'a cv::core::Mat,
        desc2: &'a cv::core::Mat,
        key_points_1: &'a Vector<KeyPoint>,
        key_points_2: &'a Vector<KeyPoint>,
    ) -> Self {
        Self {
            desc1,
            desc2,
            matches: Default::default(),
            key_points_1,
            key_points_2,
        }
    }
    pub fn compute_matches(&mut self, ratio: f32) -> Result<()> {
        let bf = cv::features2d::BFMatcher::create(cv::core::NORM_L2, false)?;
        let mut dmatches: Vector<Vector<DMatch>> = Vector::default();
        bf.knn_train_match(
            self.desc1,
            &self.desc2,
            &mut dmatches,
            2,
            &cv::core::Mat::default(),
            false,
        )?;
        let mut goods = vec![];
        for v in dmatches {
            let m = v.get(0)?;
            let m_ = v.get(1)?;
            if m.distance < ratio * m_.distance {
                goods.push(m);
                // println!("distance {}", m.distance);
            }
        }
        self.matches = goods;
        println!("good match number {}", self.matches.len());
        Ok(())
    }

    pub fn get_matching_keypoint_pair(&self) -> Vec<(KeyPoint, KeyPoint)> {
        let mut pairs: Vec<(KeyPoint, KeyPoint)> = vec![];
        for m in &self.matches {
            let query_point = self.key_points_1.get(m.query_idx as usize).unwrap();
            let train_point = self.key_points_2.get(m.train_idx as usize).unwrap();
            pairs.push((query_point, train_point));
        }
        pairs
    }
}
