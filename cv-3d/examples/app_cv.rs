use anyhow::Result;
use cv::{
    calib3d::{FM_8POINT, RANSAC},
    core::Point2f,
    core::{MatExprTraitConst, MatTraitConst, MatTraitConstManual, Vec3b, Vec3d, Vector},
    core::{Point2d, ToInputArray},
};
use cv_3d::{
    feature::FeatureProcess, filter::depth_filter::DepthFilter, frame::Frame,
    fundamental::Fundamental, utils::px2cam, *,
};
use cv_convert::{IntoCv, TryFromCv};
use kiss3d::{camera::FirstPerson, light::Light, window::Window};
use nalgebra::{DMatrix, Matrix3, Point3, Vector2};
use opencv::{self as cv, core::KeyPoint};
fn main() -> Result<()> {
    let oimg1 = cv::imgcodecs::imread("./images/1.png", cv::imgcodecs::IMREAD_COLOR)?;
    let oimg2 = cv::imgcodecs::imread("./images/2.png", cv::imgcodecs::IMREAD_COLOR)?;
    let sifts: Vec<feature::SiftFeatureProcess> = [&oimg1, &oimg2]
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
    matcher.compute_matches(0.5)?;
    let good_match = matcher.get_matching_keypoint_pair();
    let mut kpts1: Vector<Point2f> = Vector::new();
    let mut kpts2: Vector<Point2f> = Vector::new();
    let mut good_matches = vec![];
    for i in 0..good_match.len() {
        kpts1.push(Point2f::new(good_match[i].0.pt.x, good_match[i].0.pt.y));
        kpts2.push(Point2f::new(good_match[i].1.pt.x, good_match[i].1.pt.y));
        good_matches.push((
            Point2f::new(good_match[i].0.pt.x, good_match[i].0.pt.y),
            Point2f::new(good_match[i].1.pt.x, good_match[i].1.pt.y),
        ));
    }

    // let fundamental = cv::calib3d::find_fundamental_mat(
    //     &kpts1,
    //     &kpts2,
    //     cv::calib3d::FM_RANSAC,
    //     3.,
    //     0.99,
    //     1000,
    //     &mut cv::core::Mat::default(),
    // )?;

    // let fundamental = DMatrix::<f64>::try_from_cv(&fundamental)?.map(|val| val as f64);
    // let fundamental = Matrix3::from_vec(fundamental.data.as_vec().to_vec());
    // let fundamental = Fundamental(fundamental);
    // println!(" fundamental {:?}", fundamental);
    // let fundamental = Fundamental::get_fundamental_matrix(&good_matches, false);
    // // println!(" fundamental {:?}", fundamental);
    let img1 = oimg1.to_na_matrix().map(|v| v as f64);
    let img2 = oimg2.to_na_matrix().map(|v| v as f64);
    let k: Matrix3<f64> = Matrix3::new(520.9, 0., 325.1, 0., 521.0, 249.7, 0., 0., 1.);
    // let essential = fundamental.to_essential_matrix(&k, None);
    // println!("essential {:?}", essential.get_e());
    // let (r, t) = essential.find_R_T(&good_matches, None);
    // println!("r {:?}  , t {:?}", r, t);
    println!("------------------------------------");
    // let fundamental = Fundamental::get_fundamental_matrix(&good_matches, false);
    // println!(" fundamental {:?}", fundamental.0);

    // let essential = opencv::calib3d::find_essential_mat_1(
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
    // let camera = cv::core::Mat::eye(3, 3, cv::core::CV_64F)?;
    // let mut camera = camera.to_mat()?;

    // let essential = opencv::calib3d::find_essential_mat(
    //     &kpts1,
    //     &kpts2,
    //     &camera,
    //     &camera,
    //     &mut cv::core::Mat::default(),
    //     &mut cv::core::Mat::default(),
    //     FM_8POINT,
    //     0.999,
    //     0.1,
    //     &mut cv::core::Mat::default(),
    // )?;

    // println!("essential {}", DMatrix::<f64>::try_from_cv(&essential)?);
    // println!("--------------------------------");
    let fundamental = Fundamental::get_fundamental_matrix(&good_matches, true);
    let essential = fundamental.to_essential_matrix(&k, None);

    println!("essential {:?}", essential.get_e());

    // println!("essential {}", DMatrix::<f64>::try_from_cv(&esstianl)?);
    let (r, t) = essential.find_R_T(&good_matches, None);
    println!("r {:?}  , t {:?}", r, t);
    let frames = [Frame::new(&img1), Frame::new(&img2)];
    let mut depth_filter: Option<DepthFilter> = None;
    for i in 0..2 {
        if i == 0 {
            depth_filter = Some(DepthFilter::new(
                &frames[i],
                Some(3.),
                &k,
                &k,
                &r,
                &t,
                Some(3.),
            ));
        } else {
            depth_filter.as_mut().unwrap().add_frame(&frames[i]);
        }
    }

    let mut points = vec![];
    let shape = img1.shape();

    for y in 0..shape.0 {
        for x in 0..shape.1 {
            let depth = depth_filter.as_ref().unwrap().get_depth()[(y, x)];
            if depth == 3.0 {
                continue;
            }
            let pt = px2cam(&Vector2::new(x as f64, y as f64), &k);
            let pt = pt * depth;
            points.push((
                Point3::new(pt.x as f32, pt.y as f32, pt.z as f32),
                Point3::new(
                    ((oimg1.at_2d::<Vec3b>(y as i32, x as i32).unwrap()[2]) as f32 / 255.) as f32,
                    ((oimg1.at_2d::<Vec3b>(y as i32, x as i32).unwrap()[1]) as f32 / 255.) as f32,
                    ((oimg1.at_2d::<Vec3b>(y as i32, x as i32).unwrap()[0]) as f32 / 255.) as f32,
                ),
            ));
        }
    }
    println!("{:?}", points.len());
    let mut window = Window::new(" kiss 3d ");
    window.set_light(Light::StickToCamera);
    window.set_point_size(1.);
    window.set_background_color(0.9, 0.9, 0.9);
    let mut camera = FirstPerson::new(Point3::new(0., 0., 0.), Point3::new(0., 0., 1.));
    while window.render_with_camera(&mut camera) {
        for i in 0..points.len() {
            window.draw_point(&points[i].0, &points[i].1);
        }
    }
    // println! {" depth {:?}" , depth_filter.as_ref().unwrap().get_depth()}
    Ok(())
}
