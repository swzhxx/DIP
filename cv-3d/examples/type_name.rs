use std::{any::type_name, f32::consts::PI};

use nalgebra::{Matrix3, Matrix4, Vector2};
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
    let fov = radius(12.);
    let near = 0.25;
    let far = 500.;
    let z = 0.85;
    let aspect = 640. / 360.;
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

    let view_matrix = Matrix3::new()
    // let ncc = Ncc::default();
    // test_type(ncc)
}
