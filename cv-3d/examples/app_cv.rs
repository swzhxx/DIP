use anyhow::Result;
use cv::{calib3d::FM_8POINT, core::Point2f, core::ToInputArray, core::Vector};
use cv_3d::{
    feature::FeatureProcess, filter::depth_filter::DepthFilter, frame::Frame,
    fundamental::Fundamental, *,
};
use cv_convert::TryFromCv;
use kiss3d::window::Window;
use nalgebra::{DMatrix, Matrix3};
use opencv::{self as cv, core::KeyPoint};
fn main() -> Result<()> {
    let img1 = cv::imgcodecs::imread("./images/1.png", cv::imgcodecs::IMREAD_COLOR)?;
    let img2 = cv::imgcodecs::imread("./images/2.png", cv::imgcodecs::IMREAD_COLOR)?;
    let sifts: Vec<feature::SiftFeatureProcess> = [&img1, &img2]
        .iter()
        .map(|img| {
            let mut sift = feature::SiftFeatureProcess::new((*img).clone());
            sift.extract_features();
            sift
        })
        .collect();

    let matcher = feature::FeaturePointMatchBuilder::new(
        sifts[0].get_desc(),
        sifts[1].get_desc(),
        sifts[0].get_key_points(),
        sifts[1].get_key_points(),
    );

    let good_match = matcher.get_matching_keypoint_pair();
    let mut kpts1: Vector<Point2f> = Vector::new();
    let mut kpts2: Vector<Point2f> = Vector::new();
    for i in 0..good_match.len() {
        kpts1.push(Point2f::new(good_match[i].0.pt.x, good_match[i].0.pt.y));
        kpts2.push(Point2f::new(good_match[i].1.pt.x, good_match[i].1.pt.y));
    }
    let fundamental = cv::calib3d::find_fundamental_mat(
        &kpts1,
        &kpts2,
        FM_8POINT,
        3.,
        0.99,
        1000,
        &mut cv::core::Mat::default(),
    )?;
    let fundamental = DMatrix::<f32>::try_from_cv(&fundamental)?;
    let fundamental = Matrix3::from_vec(fundamental.data.as_vec().to_vec());
    let fundamental = Fundamental(fundamental);
    let img1 = img1.to_na_matrix().map(|v| v as f32);
    let img2 = img2.to_na_matrix().map(|v| v as f32);
    let k: Matrix3<f32> = Matrix3::new(520.9, 0., 325.1, 0., 521.0, 249.7, 0., 0., 1.);
    let esstinal = fundamental.to_esstianl_matrix(&k, None);

    let frames = [Frame::new(&img1), Frame::new(&img2)];
    let mut depth_filter: DepthFilter;
    for i in 0..1 {}
    Ok(())
}
