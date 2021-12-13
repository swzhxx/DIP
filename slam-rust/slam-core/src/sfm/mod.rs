mod eight_point;
use std::rc::Rc;

pub use eight_point::*;
use nalgebra::{
    ArrayStorage, Const, DMatrix, IsometryMatrix3, Matrix, Matrix1, Matrix3, Matrix3x4, Rotation3,
    Storage, Translation, Translation3, Vector2, Vector3,
};
use ndarray::{array, Array1, Array2};
use nshare::{RefNdarray1, ToNalgebra, ToNdarray2};
use num_traits::Float;

use crate::{
    optimize::LM,
    point::{Point, Point2, Point3},
    svd::{compute_min_vt_eigen_vector, sort_svd},
    triangulate::RelativeDltTriangulator,
};
use nshare::RefNdarray2;
//
pub fn get_projection_through_fundamental(fundamental: &Array2<f64>) -> Matrix3x4<f64> {
    let f_t = fundamental.t().clone().to_owned().into_nalgebra();

    let less_eigen_vector = compute_min_vt_eigen_vector(&f_t);
    let less_eigen_vector = Vector3::from_vec(less_eigen_vector).normalize();
    let b = array![
        [0., -less_eigen_vector[2], less_eigen_vector[1]],
        [less_eigen_vector[2], 0., -less_eigen_vector[0]],
        [-less_eigen_vector[1], less_eigen_vector[0], 0.]
    ];
    // let a = -&b.dot(fundamental);
    let mut ret = (-1. * &b).dot(fundamental);
    ret.push_column(less_eigen_vector.ref_ndarray1()).unwrap();

    let ret = ret.into_nalgebra();

    Matrix3x4::from_vec(ret.data.as_vec().to_vec())
}

//分解投影矩阵 得到摄像机内参数和外参数
pub fn projection_decomposition(
    projection_matrix: &Matrix3x4<f64>,
) -> (Matrix3<f64>, Matrix3x4<f64>) {
    let A = projection_matrix.slice((0, 0), (3, 3));
    let B = projection_matrix.slice((0, 3), (3, 1));
    let a1 = A.row(0);
    let a2 = A.row(1);
    let a3 = A.row(2);
    let rho = 1. / a3.norm();
    // println!("{:?}", &a1.dot(&a3));
    // println!("{:?}", &a2.dot(&a3));
    let u0: f64 = rho.powf(2.) * &a1.dot(&a3);
    let v0: f64 = rho.powf(2.) * &a2.dot(&a3);
    let a1_cross_a3 = a1.cross(&a3);
    let a2_cross_a3 = a2.cross(&a3);
    //let cos_theta = -(a1_cross_a3.dot(&a2_cross_a3) / (a1_cross_a3.norm() * a2_cross_a3.norm()));
    // let cos_theta = 0.;
    // let theta = cos_theta.acos();
    // let alpha = rho.powf(2.) * a1_cross_a3.norm() * theta.sin();
    // let beta = rho.powf(2.) * a1_cross_a3.norm() * theta.sin();

    let alpha = rho.powf(2.) * a1_cross_a3.norm();
    let beta = rho.powf(2.) * a2_cross_a3.norm();
    let r1 = a2_cross_a3.clone() / a2_cross_a3.norm();
    let r3 = a3 / a3.norm();
    let r2 = r1.cross(&r3);

    // let p1 = -alpha * (1. / theta.tan());
    // let p2 = beta / theta.sin();
    let k_inner = Matrix3::from_vec(vec![
        alpha, // if p1.is_nan() { 0. } else { p1 },
        0., u0, 0., // if p2.is_nan() { beta } else { p2 },
        beta, v0, 0., 0., 1.,
    ]);
    let mut R = Matrix3::from_element(0.);
    R.row_mut(0).copy_from(&r1);
    R.row_mut(1).copy_from(&r2);
    R.row_mut(2).copy_from(&r3);
    let T = Vector3::from_vec(
        (rho * k_inner.try_inverse().unwrap() * &B)
            .data
            .as_vec()
            .to_vec(),
    );
    let k_outer = R.clone();
    let mut k_outer = k_outer.insert_columns(3, 3, 0.);
    k_outer.column_mut(3).copy_from(&T);
    // let k_outer =
    //     IsometryMatrix3::from_parts(Translation3::new(T.x, T.y, T.z), Rotation3::from_matrix(&R));
    // let k_outer = k_outer.to_homogeneous().slice((0, 0), (3, 4)).clone_owned();

    let k_outer = Matrix3x4::from_vec(k_outer.to_owned().data.as_vec().to_vec());
    (k_inner, k_outer)
}

// 本质矩阵的分解
// 获得R,T
pub fn essential_decomposition(
    essential: &Array2<f64>,
) -> (Matrix3<f64>, Matrix3<f64>, Vector3<f64>, Vector3<f64>) {
    let essential = essential.clone().to_owned().into_nalgebra();
    let mut svd = essential.svd(true, true);
    sort_svd(&mut svd);
    let W = Matrix3::from_vec(vec![0., 1., 0., -1., 0., 0., 0., 0., 0.]);
    let U = svd.u.unwrap();
    let Vt = svd.v_t.unwrap();
    // let Tx = U * Matrix3::from_vec(vec![1., 0., 0., 0., 1., 0., 0., 0., 0.]);
    let R1 = &U * W.transpose() * Vt.transpose();
    let R1 = R1.determinant() * R1;
    let R1 = Matrix3::from_vec(R1.data.as_vec().clone());

    let R2 = &U * W.transpose() * Vt.transpose();
    let R2 = R2.determinant() * R2;
    let R2 = Matrix3::from_vec(R2.data.as_vec().clone());

    let T = U.column(2);
    let T1 = T;
    let T1 = Vector3::from_column_slice(T.as_slice());
    let T2 = -1. * &T;
    let T2 = Vector3::from_column_slice(T2.as_slice());
    (R1, R2, T1, T2)
}

pub fn find_pose_by_essential(
    essential: &Array2<f64>,
    match_points_1: &Vec<Point2<f64>>,
    match_points_2: &Vec<Point2<f64>>,
    // k1: Option<&Matrix3<f64>>,
    // k2: Option<&Matrix3<f64>>,
) -> Array2<f64> {
    let (R1, R2, T1, T2) = essential_decomposition(essential);
    // let k_identity = Matrix3::identity();
    // let k1 = k1.unwrap_or(&k_identity);
    // let k2 = k2.unwrap_or(&k_identity);
    let vec_poses = vec![(R1, T1), (R2, T2), (R2, T1), (R2, T2)];
    let counts: Vec<usize> = vec_poses
        .iter()
        .map(|(R, T)| {
            let matches = match_points_1.iter().zip(match_points_2);
            let world_ps: Vec<Option<Vector3<f64>>> = matches
                .map(|(p1, p2)| {
                    let p1 = Vector2::new(p1.x, p1.y);
                    let p2 = Vector2::new(p2.x, p2.y);
                    let triangulator = RelativeDltTriangulator::new();

                    let relative_pose = IsometryMatrix3::from_parts(
                        Translation3::from(T.clone()),
                        Rotation3::from_matrix(R),
                    )
                    .to_homogeneous();

                    let relative_pose = relative_pose.slice((0, 0), (3, 4));
                    // let pose = k2 * &relative_pose;
                    let pose =
                        Matrix3x4::from_vec(relative_pose.clone_owned().data.as_vec().to_vec());
                    // let pose = k2.to_owned() * relative_pose.to_homogeneous();
                    triangulator.triangulate_relative(&pose, &p1, &p2)
                })
                .collect();
            world_ps.iter().fold(0usize, |acc, p| {
                if *p == None {
                    acc
                } else {
                    let p = p.as_ref().unwrap();
                    if p.z > 0. && p.z > T.z {
                        acc + 1
                    } else {
                        acc
                    }
                }
            })
        })
        .collect();
    let (max_index, _) = counts.iter().enumerate().max_by(|x, y| x.cmp(y)).unwrap();
    // vec_poses[max_index].to_owned()
    let (R, T) = &vec_poses[max_index];
    // let relative_pose = R.clone().to_owned().insert_fixed_rows(3, T.to_owned());
    let relative_pose =
        IsometryMatrix3::from_parts(Translation::from(T.clone()), Rotation3::from_matrix(R))
            .to_homogeneous();
    relative_pose.ref_ndarray2().to_owned()
}

type MatchPoints<T> = Vec<Point2<T>>;

// pub fn restoration_perspective_structure<T>(
//     fundamental: &Array2<f64>,
//     match1: &MatchPoints<T>,
//     match2: &MatchPoints<T>,
//     iter: Option<usize>,
// ) -> Array2<f64>
// where
//     T: Point,
// {
//     let (mut a, b) = find_pose(fundamental);
//     let b = array![b[[2, 1]], b[[0, 2]], b[[1, 0]]];
//     // let m = b.dot(&a);
//     // let m = m.into_shape((m.len())).unwrap();
//     a.push_column(b.view());
//     let m = a;
//     let m_len = m.len();
//     let m = match m.into_shape(m_len) {
//         Ok(val) => val,
//         Err(e) => {
//             panic!("{:?}", e);
//         }
//     };

//     let match1: Vec<Point3<f64>> = match1.iter().map(|p| p.f().homogeneous()).collect();
//     let match2: Vec<Point3<f64>> = match2.iter().map(|p| p.f().homogeneous()).collect();
//     let mut lm = LM::new(
//         &match2,
//         &match1,
//         Rc::new(Box::new(|args, _, input, output| {
//             let x = args[0] * input.x + args[1] * input.y + args[2] * input.z + args[3];
//             let y = args[4] * input.x + args[5] * input.y + args[6] * input.z + args[7];
//             let z = args[8] * input.x + args[9] * input.y + args[10] * input.z + args[11];

//             ((output.x - x).powf(2.) + (output.y - y).powf(2.) + (output.z - z).powf(2.)).sqrt()
//         })),
//         Rc::new(Box::new(|args, _, input, error| -> Array1<f64> {
//             let x = args[0] * input.x + args[1] * input.y + args[2] * input.z + args[3];
//             let y = args[4] * input.x + args[5] * input.y + args[6] * input.z + args[7];
//             let z = args[8] * input.x + args[9] * input.y + args[10] * input.z + args[11];

//             let mut jaco = Array1::from_elem((args.len()), 0.);
//             jaco[0] = (1. / error.sqrt() + f64::MIN.abs()) * 2. * x * input[0];
//             jaco[1] = (1. / error.sqrt() + f64::MIN.abs()) * 2. * x * input[1];
//             jaco[2] = (1. / error.sqrt() + f64::MIN.abs()) * 2. * x * input[2];
//             jaco[3] = (1. / error.sqrt() + f64::MIN.abs()) * 2. * x;

//             jaco[4] = (1. / error.sqrt() + f64::MIN.abs()) * 2. * y * input[0];
//             jaco[5] = (1. / error.sqrt() + f64::MIN.abs()) * 2. * y * input[1];
//             jaco[6] = (1. / error.sqrt() + f64::MIN.abs()) * 2. * y * input[2];
//             jaco[7] = (1. / error.sqrt() + f64::MIN.abs()) * 2. * y;

//             jaco[8] = (1. / error.sqrt() + f64::MIN.abs()) * 2. * z * input[0];
//             jaco[8] = (1. / error.sqrt() + f64::MIN.abs()) * 2. * z * input[1];
//             jaco[10] = (1. / error.sqrt() + f64::MIN.abs()) * 2. * z * input[2];
//             jaco[11] = (1. / error.sqrt() + f64::MIN.abs()) * 2. * z;
//             jaco
//         })),
//         None,
//     );
//     let t;

//     if let Some(iter_count) = iter {
//         t = lm.optimize(&m, Some(iter_count), None);
//     } else {
//         t = lm.optimize(&m, Some(100), None);
//     }

//     t.into_shape((3, 4)).expect("t reshape failed")
// }

#[cfg(test)]
mod test {
    use nalgebra::{Matrix3, Matrix3x4};
    use ndarray::{array, Array2};

    use crate::sfm::{get_projection_through_fundamental, projection_decomposition};

    // use crate::sfm::{find_pose, };
    #[test]
    fn test_find_pose() {
        let shape = [3, 3];
        let fundamental = array![
            [
                0.00000030289221999405154,
                0.000005869970528177112,
                0.011360760530248645
            ],
            [
                0.000008181615238416967,
                -0.0000015521808080155693,
                0.0015044011091202663
            ],
            [
                -0.014145388104204511,
                0.003523120361907385,
                -0.9998280679192365
            ]
        ];
        let projection_matrix = get_projection_through_fundamental(&fundamental);
        // let projection_matrix = Matrix3x4::from_vec(vec![
        //     3.53553e2,
        //     3.39645e2,
        //     2.77744e2,
        //     -1.44946e6,
        //     -1.03528e2,
        //     2.332122e1,
        //     4.59607e2,
        //     -6.32525e5,
        //     7.07107e-1,
        //     -3.53553e-1,
        //     6.12372e-1,
        //     -9.18559e2,
        // ]);

        println!("projection {:?}", projection_matrix);
        let (k_inner, pose) = projection_decomposition(&projection_matrix);
        println!("k_inner {:?} \n pose {:?} ", k_inner, pose);
    }
}
