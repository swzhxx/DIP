mod eight_point;
use std::rc::Rc;

pub use eight_point::*;
use nalgebra::{
    Const, DMatrix, IsometryMatrix3, Matrix1, Matrix3, Rotation3, Storage, Translation,
    Translation3, Vector3,
};
use ndarray::{array, Array1, Array2};
use nshare::{RefNdarray1, ToNalgebra, ToNdarray2};

use crate::{
    optimize::LM,
    point::{Point, Point2, Point3},
    svd::{compute_min_vt_eigen_vector, sort_svd},
    triangulate::RelativeDltTriangulator,
};
use nshare::RefNdarray2;
//
pub fn get_fundamental_camera(fundamental: &Array2<f64>) -> Array2<f64> {
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
    ret
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

pub fn find_pose(
    essential: &Array2<f64>,
    match_points_1: &Vec<Point2<f64>>,
    match_points_2: &Vec<Point2<f64>>,
) -> Array2<f64> {
    let (R1, R2, T1, T2) = essential_decomposition(essential);

    let vec_poses = vec![(R1, T1), (R2, T2), (R2, T1), (R2, T2)];
    let counts: Vec<usize> = vec_poses
        .iter()
        .map(|(R, T)| {
            let matches = match_points_1.iter().zip(match_points_2);
            let world_ps: Vec<Option<Vector3<f64>>> = matches
                .map(|(p1, p2)| {
                    let p1 = Vector3::new(p1.x, p1.y, 1.);
                    let p2 = Vector3::new(p2.x, p2.y, 1.);
                    let triangulator = RelativeDltTriangulator::new();
                    let relative_pose = IsometryMatrix3::from_parts(
                        Translation3::from(T.clone()),
                        Rotation3::from_matrix(R),
                    );
                    triangulator.triangulate_relative(&relative_pose, &p1, &p2)
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
    use ndarray::{array, Array2};

    // use crate::sfm::{find_pose, };
    #[test]
    fn test_find_pose() {
        let shape = [3, 3];
        let fundamental = array![
            [4.5443750390e-6, 0.000133385576988952, -0.017984992464,],
            [
                -0.00012756570129598,
                2.26679480463767e-5,
                -0.014166784292586,
            ],
            [0.01814994639952877, 0.0041460558715090, 1.,],
        ];

        // println!("pose {:?} ", find_pose(&fundamental));
    }
}
