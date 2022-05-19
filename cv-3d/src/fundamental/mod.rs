use cv_convert::TryFromCv;
use nalgebra::{
    vector, DMatrix, DVector, Dynamic, Matrix3, Matrix3x1, Matrix3x4, Vector2, Vector3,
};
use opencv::core::{KeyPoint, Point2f};

use crate::{triangluate::RelativeDltTriangulator, ToNaVector2};

#[derive(Debug, Clone)]
pub struct Fundamental(pub Matrix3<f64>);

impl Fundamental {
    pub fn get_fundamental_matrix(
        pair_key_points: &Vec<(Point2f, Point2f)>,
        esstinal: bool,
    ) -> Self {
        let pts1: Vec<&Point2f> = pair_key_points.iter().map(|(kp, _)| return kp).collect();
        let pts2: Vec<&Point2f> = pair_key_points.iter().map(|(_, kp)| return kp).collect();
        let (n_pts1, T1) = Self::normalize(&pts1);
        let (n_pts2, T2) = Self::normalize(&pts2);

        let mut w: DMatrix<f64> = DMatrix::from_element(n_pts1.len(), 9, 0.);
        for i in 0..n_pts1.len() {
            let pt1 = n_pts1[i];
            let pt2 = n_pts2[i];

            // let row = DVector::from_vec(vec![
            //     pt1.x * pt2.x,
            //     pt1.x * pt2.y,
            //     pt1.x,
            //     pt1.y * pt2.x,
            //     pt1.y * pt2.y,
            //     pt1.x,
            //     pt2.x,
            //     pt1.y,
            //     1.,
            // ]);
            let row = DVector::from_vec(vec![
                pt1.x * pt2.x,
                pt1.y * pt2.x,
                pt2.x,
                pt1.x * pt2.y,
                pt1.y * pt2.y,
                pt2.y,
                pt1.x,
                pt1.y,
                1.,
            ]);
            w.row_mut(i).copy_from(&row.transpose());
        }
        println!("w {}", w);
        let svd = w.svd(true, true);
        let v_t = svd.v_t.unwrap();
        println!("v_t {}", v_t);
        let v = v_t.transpose();
        let f = v
            .column(v.shape().1 - 1)
            .clone_owned()
            .reshape_generic(Dynamic::new(3), Dynamic::new(3))
            .transpose();

        let mut svd = f.svd(true, true);
        println!("svd singular_values {:?}", svd.singular_values);
        *(svd.singular_values.get_mut(2).unwrap()) = 0.;
        if esstinal {
            let sigma1 = svd.singular_values[0];
            let sigma2 = svd.singular_values[1];
            let sigma = (sigma1 + sigma2) / 2.;
            *(svd.singular_values.get_mut(0).unwrap()) = sigma;
            *(svd.singular_values.get_mut(1).unwrap()) = sigma;
        }
        let f = svd.recompose().unwrap();
        println!("before f {}", f);
        let f = (T2.transpose() * &f) * &T1;
        // 这里按照f(2,2)进行处理是正确的吗?
        if !esstinal {
            Self(f / *f.get((2, 2)).unwrap())
        } else {
            Self(f)
        }
    }
    fn normalize(pts: &Vec<&Point2f>) -> (Vec<Vector3<f64>>, Matrix3<f64>) {
        let (mut u, mut v) = pts
            .iter()
            .fold((0. as f64, 0. as f64), |(mut sumx, mut sumy), kp| {
                sumx += kp.x as f64;
                sumy += kp.y as f64;
                (sumx, sumy)
            });
        let _len = pts.len();
        u = u / _len as f64;
        v = v / _len as f64;
        println!("u {:?} v {:?}", u, v);
        let d = pts.iter().fold(0. as f64, |acc, item| {
            acc + ((item.x).powf(2.) + (item.y).powf(2.)).sqrt() as f64
        });

        println!("d {:?}", d);
        let s = (2. as f64).sqrt() / d;
        let T = Matrix3::new(s, 0., -u * s, -0., s, -v * s, 0., 0., 1.);

        let pts: Vec<Vector3<f64>> = pts
            .iter()
            .map(|kp| Vector3::new(kp.x as f64, kp.y as f64, 1.))
            .map(|pt| T * pt)
            .map(|pt| pt / *pt.get(2).unwrap())
            .collect();
        println!("Normalize PT\n {:?}", pts);
        println!("Normalize T\n {}", T);
        (pts, T)
    }
}

impl Fundamental {
    pub fn to_esstianl_matrix(&self, k: &Matrix3<f64>, k2: Option<&Matrix3<f64>>) -> Essential {
        let k2 = k2.unwrap_or(k);

        Essential::new(self, k, k2)
    }
}
/// [本质矩阵分解推导](https://blog.csdn.net/kokerf/article/details/72911561)
#[derive(Debug, Clone)]
pub struct Essential {
    k1: Matrix3<f64>,
    k2: Matrix3<f64>,
    f: Fundamental,
    e: Matrix3<f64>,
}
impl Essential {
    pub fn new(f: &Fundamental, k1: &Matrix3<f64>, k2: &Matrix3<f64>) -> Self {
        let E = k2.transpose() * &(f.0) * k1;
        Self {
            k1: k1.clone_owned(),
            k2: k2.clone_owned(),
            f: f.clone(),
            e: E,
        }
    }
    pub fn get_e(&self) -> &Matrix3<f64> {
        return &self.e;
    }
    // 分解本质矩阵，得到可能的4个解
    pub fn decompose_possible_R_T(
        &self,
    ) -> (
        (Matrix3<f64>, Matrix3<f64>),
        (Matrix3x1<f64>, Matrix3x1<f64>),
    ) {
        let W = Matrix3::new(0., -1., 0., 1., 0., 0., 0., 0., 1.);
        let svd = self.e.svd(true, true);
        let U = svd.u.unwrap();
        let V_T = svd.v_t.unwrap();

        let R1 = U * &W * &V_T;
        let R1 = R1.determinant() * &R1;

        let R2 = U * &W.transpose() * &V_T;
        let R2 = R2.determinant() * &R2;

        let T1 = 1. * &U.column(2);
        let T2 = -1. * &T1;
        return ((R1, R2), (T1, T2));
    }

    // 分解本质矩阵，根据匹配的特征点，得到R，T
    pub fn find_R_T(
        &self,
        pair_match_points: &Vec<(Point2f, Point2f)>,
        count_point: Option<usize>,
    ) -> (Matrix3<f64>, Matrix3x1<f64>) {
        let ((R1, R2), (T1, T2)) = self.decompose_possible_R_T();
        let possible_r_t = vec![(R1, T1), (R1, T2), (R2, T1), (R2, T2)];
        let pair_points: Vec<(Vector2<f64>, Vector2<f64>)> = pair_match_points
            .iter()
            .map(|(kp1, kp2)| {
                let pt1 = Vector2::new(kp1.x as f64, kp1.y as f64);
                let pt2 = Vector2::new(kp2.x as f64, kp2.y as f64);
                return (pt1, pt2);
            })
            .collect();
        // 三角化判断是否深度为正数最多的那个组合
        let counts = possible_r_t
            .iter()
            .map(|(R, T)| {
                let mut p1 = self.k1.clone();
                let mut p1 = p1.insert_column(3, 0.);
                p1.column_mut(3).copy_from(&vector![0., 0., 0.]);
                let p1 = Matrix3x4::from_vec(p1.as_slice().to_vec());

                let mut p2 = R.clone();
                let mut p2 = p2.insert_column(3, 0.);
                p2.column_mut(3).copy_from(T);
                p2 = self.k2 * &p2;
                let p2 = Matrix3x4::from_vec(p2.as_slice().to_vec());

                (&pair_points).iter().fold(0u32, |acc, (pt1, pt2)| {
                    let wp =
                        RelativeDltTriangulator::triangluate_relative(&p1, &p2, pt1, pt2).unwrap();
                    if wp.z > 0. {
                        acc + 1
                    } else {
                        acc
                    }
                })
            })
            .collect::<Vec<u32>>();
        let (max_index, _) = counts
            .iter()
            .enumerate()
            .max_by(|x, y| x.1.cmp(y.1))
            .unwrap();

        return possible_r_t[max_index];
    }
}

#[cfg(test)]
mod test {
    use std::iter::zip;

    use opencv::core::Point2f;

    use crate::fundamental::Fundamental;

    #[test]
    fn test_fundamental_matrix() {
        let p1 = [
            [841.72106934, 1504.85009766],
            [555.49945068, 622.11755371],
            [862.57092285, 1008.68713379],
            [365.00683594, 712.20263672],
            [615.01538086, 828.10168457],
            [494.41290283, 370.16281128],
            [163.70254517, 378.8901062],
            [330.2401123, 736.08575439],
        ];
        let p2 = [
            [769.44750977, 1571.29101562],
            [562.44689941, 684.54248047],
            [860.09185791, 1070.83483887],
            [340.82888794, 774.70770264],
            [623.5682373, 888.65582275],
            [478.37054443, 437.3835144],
            [133.67108154, 450.15115356],
            [306.93423462, 798.13824463],
        ];

        let p1: Vec<Point2f> = p1
            .iter()
            .map(|item| return Point2f::new(item[0], item[1]))
            .collect();
        let p2 = p2
            .iter()
            .map(|item| return Point2f::new(item[0], item[1]))
            .collect::<Vec<Point2f>>();
        // let norm_p1 = Fundamental::normalize(&p1.iter().map(|v| v).collect());
        // println!("norm_p1" , norm_p1);
        let pairs: Vec<(Point2f, Point2f)> = zip(p1, p2).collect();

        let fundamental = Fundamental::get_fundamental_matrix(&pairs, false);
        println!("fundamental {}", fundamental.0);
    }
}
