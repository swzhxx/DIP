use anyhow::{anyhow, Result};

use cv::core::Point2f;
use nalgebra::{Const, DMatrix, DVector, Dynamic, Matrix3, Vector2, Vector3};
use ndarray::ArrayView3;
use opencv::core::{DMatch, KeyPoint, Vector};
use opencv::{self as cv, prelude::*};
trait FeatureProcess {
    type Output;
    fn extract_features(&mut self) -> Self::Output;
}

struct SiftFeatureProcess {
    img: cv::core::Mat,
    key_points: Vector<KeyPoint>,
    desc: cv::core::Mat,
}

impl SiftFeatureProcess {
    fn new(img: cv::core::Mat) -> Self {
        Self {
            img,
            key_points: Default::default(),
            desc: Default::default(),
        }
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
struct FeaturePointMatchBuilder<'a> {
    matches: Vec<DMatch>,
    desc1: &'a cv::core::Mat,
    desc2: &'a cv::core::Mat,
    key_points_1: &'a Vector<KeyPoint>,
    key_points_2: &'a Vector<KeyPoint>,
}

impl<'a> FeaturePointMatchBuilder<'a> {
    fn new(
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
    fn compute_matches(&mut self, ratio: f32) -> Result<()> {
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

    fn get_matching_keypoint_pair(&self) -> Vec<(KeyPoint, KeyPoint)> {
        let mut pairs: Vec<(KeyPoint, KeyPoint)> = vec![];
        for m in &self.matches {
            let query_point = self.key_points_1.get(m.query_idx as usize).unwrap();
            let train_point = self.key_points_2.get(m.train_idx as usize).unwrap();
            pairs.push((query_point, train_point));
        }
        pairs
    }
}

struct FundamentalBuilder {}

impl FundamentalBuilder {
    fn get_fundamental_matrix(pair_key_points: &Vec<(KeyPoint, KeyPoint)>) -> Matrix3<f32> {
        let pts1: Vec<&KeyPoint> = pair_key_points.iter().map(|(kp, _)| return kp).collect();
        let pts2: Vec<&KeyPoint> = pair_key_points.iter().map(|(_, kp)| return kp).collect();
        let (n_pts1, T1) = Self::normalize(&pts1);
        let (n_pts2, T2) = Self::normalize(&pts2);

        let mut w: DMatrix<f32> = DMatrix::from_element(n_pts1.len(), 9, 0.);
        for i in 0..n_pts1.len() {
            let pt1 = n_pts1[i];
            let pt2 = n_pts2[i];

            // let row = DVector::from_vec(vec![
            //     pt1.x * pt2.x,
            //     pt1.x * pt2.y,
            //     pt1.x,
            //     pt1.y * pt2.x,
            //     pt1.y * pt2.y,
            //     pt1.x,
            //     pt2.x,
            //     pt1.y,
            //     1.,
            // ]);
            let row = DVector::from_vec(vec![
                pt1.x * pt2.x,
                pt1.y * pt2.x,
                pt2.x,
                pt1.x * pt2.y,
                pt1.y * pt2.y,
                pt2.y,
                pt1.x,
                pt1.y,
                1.,
            ]);
            w.row_mut(i).copy_from(&row.transpose());
        }
        // println!("w {}", w);
        let svd = w.svd(true, true);
        let v_t = svd.v_t.unwrap();
        println!("v_t {}", v_t);
        let v = v_t.transpose();
        let f = v
            .column(v.shape().1 - 1)
            .clone_owned()
            .reshape_generic(Dynamic::new(3), Dynamic::new(3))
            .transpose();
        let mut svd = f.svd(true, true);
        println!("svd singular_values {:?}", svd.singular_values);
        *(svd.singular_values.get_mut(2).unwrap()) = 0.;
        let f = svd.recompose().unwrap();
        let f = (T2.transpose() * &f) * &T1;
        // 这里按照f(2,2)进行处理是正确的吗?
        f / *f.get((2, 2)).unwrap()
    }
    fn normalize(pts: &Vec<&KeyPoint>) -> (Vec<Vector3<f32>>, Matrix3<f32>) {
        let (u, v) = pts.iter().fold((0., 0.), |(mut sumx, mut sumy), kp| {
            sumx += kp.pt.x;
            sumy += kp.pt.y;
            (sumx, sumy)
        });
        let _len = pts.len();
        let total_offset = pts
            .iter()
            .fold(0., |acc, item| {
                acc + (item.pt.x - u).powf(2.) + (item.pt.y - v).powf(2.)
            })
            .sqrt();
        let s = (2. as f32).sqrt() / total_offset;
        let T = Matrix3::new(s, 0., -u * s, -0., s, -v * s, 0., 0., 1.);
        println!("Normalize T\n {}", T);
        let pts: Vec<Vector3<f32>> = pts
            .iter()
            .map(|kp| Vector3::new(kp.pt.x, kp.pt.y, 1.))
            .map(|pt| T * pt)
            .map(|pt| pt / *pt.get(2).unwrap())
            .collect();
        (pts, T)
    }
}

struct HomographyBuilder {}

fn main() -> Result<()> {
    let img = cv::imgcodecs::imread("./images/DSC_0480.jpg", cv::imgcodecs::IMREAD_GRAYSCALE)?;
    let img2 = cv::imgcodecs::imread("./images/DSC_0481.jpg", cv::imgcodecs::IMREAD_GRAYSCALE)?;
    let mut sift1 = SiftFeatureProcess::new(img);
    sift1.extract_features()?;
    let mut sift2 = SiftFeatureProcess::new(img2);
    sift2.extract_features()?;

    let mut feature_point_match_builder = FeaturePointMatchBuilder::new(
        &sift1.desc,
        &sift2.desc,
        &sift1.key_points,
        &sift2.key_points,
    );
    feature_point_match_builder.compute_matches(0.6)?;
    let good_pair_match_points = feature_point_match_builder.get_matching_keypoint_pair();
    println!("good_pair_match_points {}", good_pair_match_points.len());
    let fundamental_matrix = FundamentalBuilder::get_fundamental_matrix(&good_pair_match_points);
    println!("fundamental_matrix {}", fundamental_matrix);
    // let fundamental_matrix = cv::calib3d::find_fundamental_mat(
    //     &good_pair_match_points
    //         .iter()
    //         .map(|(kp, _)| Point2f::new(kp.pt.x, kp.pt.y))
    //         .collect::<Vector<Point2f>>(),
    //     &good_pair_match_points
    //         .iter()
    //         .map(|(_, kp)| Point2f::new(kp.pt.x, kp.pt.y))
    //         .collect::<Vector<Point2f>>(),
    //     cv::calib3d::FM_8POINT,
    //     0.,
    //     0.,
    //     10,
    //     &mut cv::core::Mat::default(),
    // )?;
    for i in 0..good_pair_match_points.len() {
        let (pt1, pt2) = good_pair_match_points[i];
        // let pt1 = pt1.pt;
        // let pt2 = pt2.pt;
        // let d = pt2.mul(&fundamental_matrix).mul(&pt1);
        let pt1 = Vector3::new(pt1.pt.x, pt1.pt.y, 1.);
        let pt2 = Vector3::new(pt2.pt.x, pt2.pt.y, 1.);
        let d = pt2.transpose() * &fundamental_matrix * &pt1;
        // println!("d {}", d);
    }
    Ok(())
}
