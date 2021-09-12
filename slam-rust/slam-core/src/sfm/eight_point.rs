use crate::{
    matches::{DMatch, Match},
    point::{Point2, Point3},
};
use ndarray::{Array1, Array2, Axis};
use nshare::ToNalgebra;
use num_traits::{AsPrimitive, Float, Num};

/// **Eight Point Algorithm**
///
/// [八点法](https://www.cnblogs.com/wangguchangqing/p/8214032.html)
pub struct EightPoint<'a> {
    match_points_1: &'a Vec<Point2<usize>>,
    match_points_2: &'a Vec<Point2<usize>>,
}

impl<'a> EightPoint<'_> {
    pub fn new(
        match_points_1: &'a Vec<Point2<usize>>,
        match_points_2: &'a Vec<Point2<usize>>,
    ) -> EightPoint<'a> {
        EightPoint {
            match_points_1,
            match_points_2,
        }
    }

    /// 8点法计算基础矩阵
    fn calc_fundamental(
        &self,
        points1: &Vec<Point3<f64>>,
        points2: &Vec<Point3<f64>>,
    ) -> Option<Array2<f64>> {
        let len = points1.len().min(points2.len());
        let f: Option<Array2<f64>> = None;
        if len < 8 {
            f
        } else {
            let mut w = Array2::<f64>::zeros((0, 9));
            for k in 0..len {
                let mp1 = &points1[k];
                let mp2 = &points2[k];
                w.push(
                    Axis(0),
                    Array1::<f64>::from_vec(vec![
                        (mp2.x * mp1.x) as f64,
                        (mp2.x * mp1.y) as f64,
                        (mp2.x as f64),
                        (mp2.y * mp1.x) as f64,
                        (mp2.y * mp1.y) as f64,
                        mp2.y as f64,
                        mp1.x as f64,
                        mp1.y as f64,
                        1.,
                    ])
                    .view(),
                )
                .unwrap();
            }

            let svd = w.view().into_nalgebra().svd(false, true);
            let v_t = svd.v_t.unwrap();
            let mut f = v_t.column(8);
            let mut f_svd = f.svd(true, true);
            f_svd.singular_values[2] = 0.;
            let f_bar = f_svd.recompose().unwrap();
            let mut res: Vec<f64> = vec![];
            for val in f_bar.iter() {
                res.push(*val);
            }
            let f_bar: Array2<f64> = Array2::from_shape_vec((3, 3), res).unwrap();
            Some(f_bar)
        }
    }

    pub fn find_fundamental(self) -> Option<Array2<f64>> {
        let points1 = self
            .match_points_1
            .iter()
            .map(|p| Point3::new(p.x as f64, p.y as f64, 1.))
            .collect();
        let points2 = self
            .match_points_2
            .iter()
            .map(|p| Point3::new(p.x as f64, p.y as f64, 1.))
            .collect();
        self.calc_fundamental(&points1, &points2)
    }

    /// 归一化8点法计算基础矩阵
    pub fn normalize_find_fundamental(&mut self) -> Option<Array2<f64>> {
        let to_float = |points: &Vec<Point2<usize>>| -> Vec<Point2<f64>> {
            points
                .iter()
                .map(|p| Point2::<f64>::new(p.x as f64, p.y as f64))
                .collect()
        };

        let points1 = to_float(&self.match_points_1);
        let points2 = to_float(&self.match_points_2);
        let h1 = self.get_normalize_translate_matrix(&points1);
        let h2 = self.get_normalize_translate_matrix(&points2);

        let build_normalize_points =
            |points: &Vec<Point2<f64>>, t: &Array2<f64>| -> Vec<Point3<f64>> {
                points
                    .iter()
                    .map(|p| {
                        let p3 = p.homogeneous();
                        let p = t.dot(&p3.data);
                        Point3::<f64>::new(p[0], p[1], p[2])
                    })
                    .collect()
            };
        let normalize_points1 = build_normalize_points(&points1, &h1);
        let normalize_points2 = build_normalize_points(&points2, &h2);

        if let Some(fundmatental) = self.calc_fundamental(&normalize_points1, &normalize_points2) {
            Some(h2.t().dot(&fundmatental).dot(&h1))
        } else {
            None
        }
    }

    fn get_normalize_translate_matrix(&self, points: &Vec<Point2<f64>>) -> Array2<f64> {
        let u_bar = points.iter().fold(0., |a, b| a + b.x);
        let v_bar = points.iter().fold(0., |a, b| a + b.y);
        let len = points.len() as f64;
        let divisor = points.iter().fold(0., |prev, p| {
            prev + (p.x as f64 - u_bar) + (p.y as f64 - v_bar)
        });
        let s = (2. * len).sqrt() / divisor.sqrt();

        s * Array2::from_shape_vec((3, 3), vec![1., 0., -u_bar, 0., 1., -v_bar, 0., 0., 1. / s])
            .unwrap()
    }
}
