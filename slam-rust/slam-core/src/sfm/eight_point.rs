use crate::svd::{compute_min_vt_eigen_vector, sort_svd};
use crate::{
    matches::{DMatch, Match},
    point::{Point2, Point3},
};
use nalgebra::{Const, Dim, Dynamic, Matrix3, Matrix3xX};
use ndarray::{array, Array1, Array2, Axis};
use nshare::{RefNdarray2, ToNalgebra};
use num_traits::{AsPrimitive, Float, Num};

/// **Eight Point Algorithm**
///
/// [八点法](https://www.cnblogs.com/wangguchangqing/p/8214032.html)
pub struct EightPoint<'a> {
    match_points_1: &'a Vec<Point2<f64>>,
    match_points_2: &'a Vec<Point2<f64>>,
}

impl<'a> EightPoint<'_> {
    pub fn new(
        match_points_1: &'a Vec<Point2<f64>>,
        match_points_2: &'a Vec<Point2<f64>>,
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

            let f = compute_min_vt_eigen_vector(&w.view().into_nalgebra().clone_owned());

            let f = Array2::from_shape_vec((3, 3), f).unwrap();
            let f = f.into_nalgebra();

            let mut f_svd = f.svd(true, true);
            // println!("f_svd {:?}", f_svd);
            sort_svd(&mut f_svd);
            // println!("f_svd {:?}", f_svd);
            f_svd.singular_values[2] = 0.;

            // println!("f_svd u {:?}", f_svd.u);
            // println!("f_svd v_t {:?}", f_svd.v_t);
            let f_bar = f_svd.recompose().unwrap();
            // let f_bar = f_svd.u.unwrap() * f_svd.singular_values * f_svd.v_t.unwrap();
            let f_bar = f_bar.ref_ndarray2().to_owned();

            Some(f_bar)
        }
    }

    pub fn find_fundamental(self) -> Option<Array2<f64>> {
        let points1 = self
            .match_points_1
            .iter()
            .map(|p| p.homogeneous())
            .collect();
        let points2 = self
            .match_points_2
            .iter()
            .map(|p| p.homogeneous())
            .collect();
        self.calc_fundamental(&points1, &points2)
    }

    /// 归一化8点法计算基础矩阵
    pub fn normalize_find_fundamental(&mut self) -> Option<Array2<f64>> {
        let points1 = &self.match_points_1;
        let points2 = &self.match_points_2;
        let h1 = self.get_normalize_translate_matrix(points1);
        let h2 = self.get_normalize_translate_matrix(points2);

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
            let f0 = h2.t().dot(&fundmatental).dot(&h1);
            let f0 = 1. / f0[[2, 2]] * f0;
            Some(f0)
        } else {
            None
        }
    }

    fn get_normalize_translate_matrix(&self, points: &Vec<Point2<f64>>) -> Array2<f64> {
        let len = points.len() as f64;
        let u_bar = points.iter().fold(0., |a, b| a + b.x) / len;
        let v_bar = points.iter().fold(0., |a, b| a + b.y) / len;

        let divisor = points.iter().fold(0., |prev, p| {
            prev + (p.x as f64 - u_bar).powf(2.) + (p.y as f64 - v_bar).powf(2.)
        });
        let s = ((2. * len) / divisor).sqrt();

        Array2::from_shape_vec(
            (3, 3),
            vec![s, 0., -u_bar * s, 0., s, -v_bar * s, 0., 0., 1.],
        )
        .unwrap()
    }
}

#[cfg(test)]
mod test {
    use ndarray::array;

    use crate::{point::Point2, sfm::EightPoint};

    #[test]
    fn test_nomalize_find_fundamental() {
        use crate::{point::Point2, sfm::EightPoint};
        let points1: Vec<(f64, f64)> = vec![
            (473.000000, 395.000000),
            (278.000000, 301.000000),
            (219.000000, 336.000000),
            (260.000000, 208.000000),
            (225.000000, 208.000000),
            (323.000000, 206.000000),
            (236.000000, 265.000000),
            (335.000000, 262.000000),
            (301.000000, 262.000000),
            (35.000000, 320.000000),
            (403.000000, 321.000000),
            (424.000000, 99.000000),
            (443.000000, 372.000000),
            (438.000000, 331.000000),
            (413.000000, 322.000000),
            (188.000000, 182.000000),
            (296.000000, 244.000000),
            (499.000000, 296.000000),
            (485.000000, 344.000000),
            (461.000000, 346.000000),
            (460.000000, 111.000000),
            (410.000000, 174.000000),
            (41.000000, 261.000000),
            (54.000000, 259.000000),
            (392.000000, 315.000000),
            (342.000000, 299.000000),
            (243.000000, 303.000000),
            (232.000000, 246.000000),
            (266.000000, 245.000000),
            (330.000000, 243.000000),
            (373.000000, 171.000000),
            (362.000000, 190.000000),
            (307.000000, 300.000000),
            (289.000000, 207.000000),
            (377.000000, 378.000000),
            (381.000000, 130.000000),
            (270.000000, 264.000000),
        ];
        let points2: Vec<(f64, f64)> = vec![
            (358.000000, 423.000000),
            (247.000000, 306.000000),
            (185.000000, 333.000000),
            (261.928345, 233.894287),
            (228.500000, 234.000000),
            (325.812714, 232.719757),
            (221.052612, 280.881195),
            (313.185333, 279.474823),
            (282.893768, 278.836487),
            (25.000000, 177.000000),
            (346.000000, 331.000000),
            (453.000000, 294.000000),
            (351.000000, 382.000000),
            (363.000000, 375.000000),
            (354.000000, 332.000000),
            (201.000000, 214.000000),
            (285.000000, 264.000000),
            (402.000000, 445.000000),
            (380.000000, 432.000000),
            (368.826630, 406.454346),
            (461.000000, 355.000000),
            (408.000000, 291.000000),
            (38.000000, 280.000000),
            (51.000000, 280.000000),
            (342.000000, 315.000000),
            (305.000000, 306.000000),
            (216.000000, 308.000000),
            (223.295013, 264.546631),
            (255.373993, 264.654572),
            (318.256165, 261.628693),
            (393.000000, 214.000000),
            (373.000000, 217.000000),
            (274.000000, 306.000000),
            (292.833252, 233.668701),
            (294.000000, 387.000000),
            (416.000000, 223.000000),
            (253.000000, 281.000000),
        ];
        let create_point = |p: (f64, f64)| -> Point2<f64> { Point2::new(p.0, p.1) };
        let points1: Vec<Point2<f64>> = points1.into_iter().map(create_point).collect();
        let points2: Vec<Point2<f64>> = points2.into_iter().map(create_point).collect();

        let mut ep = EightPoint::new(&points1, &points2);
        let fundamental = ep.normalize_find_fundamental();
        println!("normalize fundamental matrix {:?}", fundamental);
        let fundamental = ep.find_fundamental();
        println!("fundamental matrix {:?}", fundamental);
    }

    #[test]

    fn randomized_test() {
        use nalgebra::{
            matrix, Const, IsometryMatrix3, Matrix3, OPoint, Point2, Point3, Rotation3, Vector2,
            Vector3,
        };
        const SAMPLE_POINTS: usize = 16;
        const RESIDUAL_THRESHOLD: f64 = 1e-4;
        const ROT_MAGNITUDE: f64 = 0.2;
        const POINT_BOX_SIZE: f64 = 2.0;
        const POINT_DISTANCE: f64 = 3.0;
        fn some_test_data() -> (IsometryMatrix3<f64>, Vec<Point2<f64>>, Vec<Point2<f64>>) {
            let camera = Matrix3::new(520.9, 0., 320., 0., 521.0, 320.1, 0., 0., 1.);
            let relative_pose = IsometryMatrix3::from_parts(
                Vector3::new_random().into(),
                Rotation3::new(Vector3::new_random() * std::f64::consts::PI * 2.0 * ROT_MAGNITUDE),
            );
            // println!("relative_pose {:?}", &relative_pose);
            let a_points: Vec<Point3<f64>> = (0..SAMPLE_POINTS)
                .map(|_| {
                    let mut a: Point3<f64> = Point3::from(Vector3::new_random() * POINT_BOX_SIZE);
                    a.x -= 0.5 * POINT_BOX_SIZE;
                    a.y -= 0.5 * POINT_BOX_SIZE;
                    a.z += POINT_DISTANCE;
                    // camera * &a
                    a
                })
                .collect();

            let b_points: Vec<Point3<f64>> = a_points
                .clone()
                .iter()
                .map(|ap| relative_pose.transform_point(&ap))
                .collect();
            // println!("a points {:?}", a_points);
            // println!("b points {:?}", b_points);
            let world2px = |point: &Point3<f64>, camera: &Matrix3<f64>| -> Point2<f64> {
                let cx = camera[(0, 2)];
                let cy = camera[(1, 2)];
                let fx = camera[(0, 0)];
                let fy = camera[(1, 1)];
                Point2::from(Vector2::new(
                    point.x * fx / point.z + cx,
                    point.y * fy / point.z + cy,
                ))
            };
            let kps_a: Vec<Point2<f64>> = a_points
                .clone()
                .iter()
                .map(|a| world2px(a, &camera))
                .collect();
            let kps_b: Vec<Point2<f64>> = b_points
                .clone()
                .iter()
                .map(|b| world2px(b, &camera))
                .collect();
            // println!("kps_a {:?}", &kps_a);
            // println!("kps_b {:?}", &kps_b);
            (relative_pose, kps_a, kps_b)
        }
        fn run_round() -> bool {
            let (relative_pose, kps_a, kps_b) = some_test_data();
            let kps_a = kps_a
                .iter()
                .map(|p| crate::point::Point2::new(p.x, p.y))
                .collect();
            let kps_b = kps_b
                .iter()
                .map(|p| crate::point::Point2::new(p.x, p.y))
                .collect();
            let mut ep = EightPoint::new(&kps_a, &kps_b);
            let fundamental = ep.normalize_find_fundamental().unwrap();
            println!("fundamental {:?}", fundamental);
            for i in 0..kps_a.len() {
                let kpa = &kps_a[i];
                let kpb = &kps_b[i];
                let kpa = array![kpa.x, kpa.y, 1.];
                let kpb = array![kpb.x, kpb.y, 1.];
                let residual = (kpb.t().dot(&fundamental).dot(&kpa)).abs();
                if residual > RESIDUAL_THRESHOLD {
                    println!("residual {:?}", residual);
                    return false;
                }
            }
            return true;
        }
        let successes = (0..1000).filter(|_| run_round()).count();
        println!("success {:?}", successes);
        assert!(successes > 950)
    }
}
