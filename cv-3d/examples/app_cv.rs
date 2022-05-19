use anyhow::Result;
use cv::{
    calib3d::{FM_8POINT, RANSAC},
    core::Point2f,
    core::Vector,
    core::{Point2d, ToInputArray},
};
use cv_3d::{
    feature::FeatureProcess, filter::depth_filter::DepthFilter, frame::Frame,
    fundamental::Fundamental, *,
};
use cv_convert::{IntoCv, TryFromCv};
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
            sift.extract_features().unwrap();

            sift
        })
        .collect();

    let mut matcher = feature::FeaturePointMatchBuilder::new(
        sifts[0].get_desc(),
        sifts[1].get_desc(),
        sifts[0].get_key_points(),
        sifts[1].get_key_points(),
    );
    matcher.compute_matches(0.6)?;
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

    let fundamental = DMatrix::<f64>::try_from_cv(&fundamental)?.map(|val| val as f32);
    let fundamental = Matrix3::from_vec(fundamental.data.as_vec().to_vec());
    let fundamental = Fundamental(fundamental);
    println!(" fundamental {:?}", fundamental);
    // let fundamental = Fundamental::get_fundamental_matrix(&good_match, false);
    // println!(" fundamental {:?}", fundamental);

    let img1 = img1.to_na_matrix().map(|v| v as f32);
    let img2 = img2.to_na_matrix().map(|v| v as f32);
    let k: Matrix3<f32> = Matrix3::new(520.9, 0., 325.1, 0., 521.0, 249.7, 0., 0., 1.);
    let esstinal = fundamental.to_esstianl_matrix(&k, None);
    println!("esstinal {:?}", esstinal.get_e());
    // let esstianl = cv::calib3d::find_essential_mat_1(
    //     &kpts1,
    //     &kpts2,
    //     521.,
    //     Point2d::new(325.1, 249.7),
    //     RANSAC,
    //     0.999,
    //     1.,
    //     1000,
    //     &mut cv::core::Mat::default(),
    // )?;
    // println!("esstinal {}", DMatrix::<f64>::try_from_cv(&esstianl)?);
    // let (r, t) = esstinal.find_R_T(&good_match, None);
    // println!("r {:?}  , t {:?}", r, t);
    // let frames = [Frame::new(&img1), Frame::new(&img2)];
    // let mut depth_filter: Option<DepthFilter> = None;
    // for i in 0..1 {
    //     if i == 0 {
    //         depth_filter = Some(DepthFilter::new(
    //             &frames[0],
    //             Some(3.),
    //             &k,
    //             &k,
    //             &r,
    //             &t,
    //             Some(3.),
    //         ));
    //     } else {
    //         if !depth_filter.is_none() {
    //             depth_filter.as_mut().unwrap().add_frame(&frames[1]);
    //         }
    //     }
    // }
    Ok(())
}
