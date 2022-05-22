use std::{any::type_name, f32::consts::PI};

use cv_3d::utils::px2cam;
use nalgebra::{Matrix3, Matrix4, Vector2, Vector4};
// use cv_3d::block_match::Ncc;
use num_traits::ToPrimitive;

fn test_type<T>(_: T) {
    println!("{:?}", { type_name::<T>() });
}

fn radius(angle: f32) -> f32 {
    return angle / 180. * PI;
}

fn main() {
    let a = 2;
    // println!("{}", a.to_f32().unwrap());
    let view_width = 640.;
    let view_height = 360.;
    let fov = radius(12.);
    let near = 0.25;
    let far = 500.;
    let depth = 0.85;
    let aspect = 1.;
    let image = Vector2::new(44., 36.);

    let cot_fov_div_2 = 1. / (fov / 2.).tan();
    let presp_matrix = Matrix4::new(
        cot_fov_div_2 / (aspect * near),
        0.,
        0.,
        0.,
        0.,
        cot_fov_div_2 / near,
        0.,
        0.,
        0.,
        0.,
        near + far / (near - far),
        -2. * near * far / (near - far),
        0.,
        0.,
        1.,
        0.,
    );

    let view_port_matrix = Matrix4::new(
        view_width / 2.,
        0.,
        (view_width / 2.),
        0.,
        0.,
        view_height / 2.,
        (view_height) / 2.,
        0.,
        0.,
        0.,
        1.,
        0.,
        0.,
        0.,
        0.,
        1.,
    );

    let image_homo = Vector4::new(image.x * depth, image.y * depth, depth, 1.);
    let pt_3d = view_port_matrix.try_inverse().unwrap()
        * &presp_matrix.try_inverse().unwrap()
        * &image_homo;
    println!("pt_3d {:?}", pt_3d / pt_3d.w);
    let k = Matrix3::new(
        view_width as f64,
        0.,
        0.,
        0.,
        view_width as f64,
        0.,
        0.,
        0.,
        1.,
    );
    let pt = px2cam(&Vector2::new(42., 34.), &k);
    println!("pt {:?}", pt);
    let pt_3d = pt * (depth as f64);
    println!("pt_3d {:?}", pt_3d)
    // let vp = view_matrix * presp_matrix ;
    // let ncc = Ncc::default();
    // test_type(ncc)
}
