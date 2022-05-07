use nalgebra::{DMatrix, Matrix3x4, Vector2, Vector3, Vector4};

/// [SVD三角化](https://blog.csdn.net/qq_37611824/article/details/93210012)
pub struct RelativeDltTriangulator;

impl RelativeDltTriangulator {
    pub fn triangluate_relative(
        p1: &Matrix3x4<f32>,
        p2: &Matrix3x4<f32>,
        a: &Vector2<f32>,
        b: &Vector2<f32>,
    ) -> Option<Vector3<f32>> {
        let mut design = DMatrix::<f32>::zeros(4, 4);
        design.row_mut(0).copy_from(&(-p1.row(1) + a.y * p1.row(2)));
        design.row_mut(1).copy_from(&(p1.row(0) - a.x * p1.row(2)));
        design.row_mut(2).copy_from(&(-p2.row(1) + b.y * p2.row(2)));
        design.row_mut(3).copy_from(&(p2.row(0) - b.x * p2.row(2)));
        let svd = design.svd(true, true);
        let point = Vector4::from_column_slice(svd.v_t.unwrap().transpose().column(3).as_slice());
        let point = &point / point.w;
        return Some(point.xyz());
    }
}
