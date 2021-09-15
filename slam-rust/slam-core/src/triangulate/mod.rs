use std::convert::{TryFrom, TryInto};

use ndarray::{array, s, Array2, Axis};
use nshare::{ToNalgebra, ToNdarray1, ToNdarray2};

use crate::point::{Point, Point3};

pub type TransformPoint = Vec<(Point3<f64>, Array2<f64>, Point3<f64>)>;

pub struct Triangulate<'a> {
    pairs: &'a TransformPoint,
}

impl<'a> Triangulate<'_> {
    pub fn new(pairs: &'a TransformPoint) -> Triangulate {
        Triangulate { pairs }
    }

    pub fn batch_svd_solve(&self) -> Vec<Point3<f64>> {
        let mut points = vec![];
        for item in self.pairs {
            let point = self.solve(item);
            points.push(point);
        }
        points
    }
    /// svd 求解
    pub fn solve(&self, pair: &(Point3<f64>, Array2<f64>, Point3<f64>)) -> Point3<f64> {
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
