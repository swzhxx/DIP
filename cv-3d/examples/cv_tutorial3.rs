use anyhow::anyhow;
use anyhow::Result;
use cv::core::CV_8U;
use cv::imgproc::COLOR_BGR2GRAY;
use cv_convert::{FromCv, IntoCv, TryFromCv, TryIntoCv};
use image::RgbImage;
use nalgebra as na;
use opencv::{self as cv, prelude::*};
fn main() -> Result<()> {
    let cv_point = cv::core::Point2f::new(1.0, 3.0);
    let na_points = na::Point2::<f32>::from_cv(&cv_point);

    let img = cv::imgcodecs::imread("./images/1.png", COLOR_BGR2GRAY)?;

    println!("img {:?}", img);
    let mat_size = img.mat_size();
    println!("mat_size {:?}", mat_size);
    println!(" channels {:?}", img.channels());

    let mut m = Mat::eye(mat_size[0], mat_size[1], CV_8U)?;
    let mut m = m.c();
    cv::imgproc::cvt_color(&img, &mut m, COLOR_BGR2GRAY, 0)?;
    println!("{:?}", m);
    let na_matrix = na::DMatrix::<u8>::try_from_cv(m)?;
    println!("ma_matrix {}", na_matrix);
    let img = cv::imgcodecs::imread("./images/1.png", cv::imgcodecs::IMREAD_COLOR)?;
    let mat_size = img.mat_size();
    println!("mat_size {:?}", mat_size);
    println!(" channels {:?}", img.channels());
    Ok(())
}
