use std::convert::{TryFrom, TryInto};

use nalgebra::{
    DMatrix, DimName, Isometry3, IsometryMatrix3, Matrix3x4, Matrix4, RowVector4, Vector2, Vector3,
    Vector4,
};
use ndarray::{array, s, Array2, Axis};
use nshare::{ToNalgebra, ToNdarray1, ToNdarray2};

use crate::{
    point::{Point, Point3},
    svd::compute_min_vt_eigen_vector,
};

pub type TransformPoint = Vec<(Point3<f64>, Array2<f64>, Point3<f64>)>;

pub struct Triangulate {
    // pairs: &'a TransformPoint,
}

impl Triangulate {
    pub fn new() -> Self {
        Triangulate {}
    }

    // pub fn batch_svd_solve(&self) -> Vec<Point3<f64>> {
    //     let mut points = vec![];
    //     for item in self.pairs {
    //         let point = self.solve(item);
    //         points.push(point);
    //     }
    //     points
    // }
    /// svd 求解
    pub fn solve(&self, pair: &(&Point3<f64>, &Array2<f64>, &Point3<f64>)) -> Point3<f64> {
        let mut left_matrix = Array2::<f64>::from_shape_vec((0, 3), vec![]).unwrap();
        // for item in self.pair {
        let (point_1, transform_matrix, point_2) = pair;
        let eye = Array2::<f64>::eye(3);
        let eye = eye.view();

        left_matrix
            .push(
                Axis(0),
                (&eye.slice(s![1, ..]) + point_1.y * &eye.slice(s![2, ..])).view(),
            )
            .unwrap();
        left_matrix
            .push(
                Axis(0),
                (&eye.slice(s![1, ..]) - point_1.x * &eye.slice(s![2, ..])).view(),
            )
            .unwrap();

        left_matrix
            .push(
                Axis(0),
                (transform_matrix.slice(s![1, ..]).to_owned()
                    + point_2.y * transform_matrix.slice(s![2, ..]).to_owned())
                .view(),
            )
            .unwrap();
        left_matrix
            .push(
                Axis(0),
                (transform_matrix.slice(s![1, ..]).to_owned()
                    - point_2.x * transform_matrix.slice(s![2, ..]).to_owned())
                .view(),
            )
            .unwrap();
        // }
        let left_matrix = left_matrix.into_nalgebra();
        let svd = left_matrix.svd(false, true);
        let x = svd.v_t.unwrap();
        let x: Point3<f64> = x.column(2).into_ndarray1().to_owned().try_into().unwrap();
        x
    }
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct RelativeDltTriangulator {
    epsilon: f64,
    max_iterations: usize,
}

impl RelativeDltTriangulator {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn epsilon(self, epsilon: f64) -> Self {
        Self { epsilon, ..self }
    }
    pub fn max_iterations(self, max_iterations: usize) -> Self {
        Self {
            max_iterations,
            ..self
        }
    }
}
impl Default for RelativeDltTriangulator {
    fn default() -> Self {
        Self {
            epsilon: 1e-12,
            max_iterations: 1000,
        }
    }
}

impl RelativeDltTriangulator {
    pub fn triangulate_relative(
        &self,
        relative_pose: &Matrix3x4<f64>,
        a: &Vector2<f64>,
        b: &Vector2<f64>,
    ) -> Option<Vector3<f64>> {
        let pose = relative_pose;
        let mut design: DMatrix<f64> = DMatrix::<f64>::zeros(4, 4);
        let eye = Matrix3x4::identity();
        design
            .row_mut(0)
            .copy_from(&(-eye.row(1) + a.y * eye.row(2)));
        design
            .row_mut(1)
            .copy_from(&(-eye.row(0) + a.x * eye.row(2)));
        design
            .row_mut(2)
            .copy_from(&(-pose.row(1) + b.y * pose.row(2)));
        design
            .row_mut(3)
            .copy_from(&(-pose.row(0) + b.x * pose.row(2)));
        // let design = DMatrix::cop
        let x = compute_min_vt_eigen_vector(&design);
        let x = Vector4::from_vec(x);
        let x = &x / x[(3, 0)];
        Some(x.xyz())
    }
}
