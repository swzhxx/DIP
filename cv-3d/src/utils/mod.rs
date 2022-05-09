use nalgebra::{Matrix3, Vector2, Vector3};

/// 像素坐标转化到归一化的摄像机坐标
pub fn px2cam(pt: &Vector2<u32>, camera_inner_matrix: &Matrix3<f32>) -> Vector3<f32> {
    let cx = camera_inner_matrix[(0, 2)];
    let cy = camera_inner_matrix[(1, 2)];
    let fx = camera_inner_matrix[(0, 1)];
    let fy = camera_inner_matrix[(1, 1)];

    Vector3::new((pt.x as f32 - cx) / fx, (pt.y as f32 - cy) / fy, 1.)
}

/// 摄像机坐标转化到像素坐标
pub fn cam2px(pt: &Vector3<f32>, camera_inner_matrix: &Matrix3<f32>) -> Vector2<f32> {
    let cx = camera_inner_matrix[(0, 2)];
    let cy = camera_inner_matrix[(1, 2)];
    let fx = camera_inner_matrix[(0, 1)];
    let fy = camera_inner_matrix[(1, 1)];
    Vector2::new(pt.x * fx / pt.z + cx, pt.y * fy / pt.z + cy)
}
