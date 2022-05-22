use nalgebra::{DMatrix, DVector, Matrix3, Scalar, Vector2, Vector3};
use num_traits::ToPrimitive;

/// 像素坐标转化到归一化的摄像机坐标
pub fn px2cam<T>(pt: &Vector2<T>, camera_inner_matrix: &Matrix3<f64>) -> Vector3<f64>
where
    T: ToPrimitive + Scalar,
{
    let cx = camera_inner_matrix[(0, 2)];
    let cy = camera_inner_matrix[(1, 2)];
    let fx = camera_inner_matrix[(0, 0)];
    let fy = camera_inner_matrix[(1, 1)];

    let x = pt.x.to_f64().unwrap();
    let y = pt.y.to_f64().unwrap();
    Vector3::new((x - cx) / fx, (y - cy) / fy, 1.)
}

/// 摄像机坐标转化到像素坐标
pub fn cam2px(pt: &Vector3<f64>, camera_inner_matrix: &Matrix3<f64>) -> Vector2<f64> {
    let cx = camera_inner_matrix[(0, 2)];
    let cy = camera_inner_matrix[(1, 2)];
    let fx = camera_inner_matrix[(0, 1)];
    let fy = camera_inner_matrix[(1, 1)];
    Vector2::new(pt.x * fx / pt.z + cx, pt.y * fy / pt.z + cy)
}

pub fn compute_min_vt_eigen_vector(m: &DMatrix<f64>) -> DVector<f64> {
    let shape = m.shape();
    if shape.0 != shape.1 {
        let v = m.transpose() * m;
        let svd = v.svd(true, false);

        let u = svd.u.expect("u failed");
        let column = DVector::from_column_slice(&u.column(u.shape().1 - 1).data.into_slice());
        column
    } else {
        let m = m.clone().to_owned();
        let mut svd = m.svd(true, true);
        let u = svd.u.expect("u failed");
        let min_eigen_vector =
            DVector::from_column_slice(u.column(u.shape().1 - 1).to_owned().data.into_slice());
        min_eigen_vector
    }
}
