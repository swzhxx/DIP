use nalgebra::{Matrix3, Scalar, Vector2, Vector3};
use num_traits::ToPrimitive;

/// 像素坐标转化到归一化的摄像机坐标
pub fn px2cam<T>(pt: &Vector2<T>, camera_inner_matrix: &Matrix3<f32>) -> Vector3<f32>
where
    T: ToPrimitive + Scalar,
{
    let cx = camera_inner_matrix[(0, 2)];
    let cy = camera_inner_matrix[(1, 2)];
    let fx = camera_inner_matrix[(0, 1)];
    let fy = camera_inner_matrix[(1, 1)];

    let x = pt.x.to_f32().unwrap();
    let y = pt.y.to_f32().unwrap();
    Vector3::new((x - cx) / fx, (y - cy) / fy, 1.)
}

/// 摄像机坐标转化到像素坐标
pub fn cam2px(pt: &Vector3<f32>, camera_inner_matrix: &Matrix3<f32>) -> Vector2<f32> {
    let cx = camera_inner_matrix[(0, 2)];
    let cy = camera_inner_matrix[(1, 2)];
    let fx = camera_inner_matrix[(0, 1)];
    let fy = camera_inner_matrix[(1, 1)];
    Vector2::new(pt.x * fx / pt.z + cx, pt.y * fy / pt.z + cy)
}