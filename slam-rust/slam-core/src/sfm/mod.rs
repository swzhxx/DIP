mod eight_point;
use std::rc::Rc;

pub use eight_point::*;
use nalgebra::{
    ArrayStorage, Const, DMatrix, IsometryMatrix3, Matrix, Matrix1, Matrix3, Matrix3x4, Rotation3,
    Storage, Translation, Translation3, Vector2, Vector3, QR,
};
use ndarray::{array, s, Array1, Array2};
use nshare::{RefNdarray1, ToNalgebra, ToNdarray2};
use num_traits::{Float, Pow};

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
    // let less_eigen_vector = less_eigen_vector / less_eigen_vector.z;
    let b = array![
        [0., -less_eigen_vector[2], less_eigen_vector[1]],
        [less_eigen_vector[2], 0., -less_eigen_vector[0]],
        [-less_eigen_vector[1], less_eigen_vector[0], 0.]
    ];
    // let a = -&b.dot(fundamental);
    let mut ret = (&b).dot(&fundamental.t());
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
    println!("cross {:?}", &a1_cross_a3.dot(&a2_cross_a3));
    println!("norm {:?}", &a1_cross_a3.norm() * a2_cross_a3.norm());
    let cos_theta = -(a1_cross_a3.dot(&a2_cross_a3) / (a1_cross_a3.norm() * a2_cross_a3.norm()));
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

/// qr分解得到摄像机矩阵[https://www.mathworks.com/matlabcentral/answers/472171-how-to-calculate-the-camera-intrinsics-k-rotation-matrix-r-and-translation-vector-t-through-the-ca]
pub fn compute_projection_qr_decomposition(p: &Matrix3x4<f64>) -> (Matrix3<f64>, Matrix3x4<f64>) {
    let m = p.slice((0, 0), (3, 3));

    let c = -m[(2, 2)] / (m[(2, 2)].powf(2.) + m[(2, 1)].powf(2.)).sqrt();
    let s = m[(2, 1)] / (m[(2, 2)].powf(2.) + m[(2, 1)].powf(2.)).sqrt();
    let Qx = Matrix3::from_vec(vec![1., 0., 0., 0., c, s, 0., -s, c]);
    let R = &m * &Qx;
    let c = R[(2, 2)] / (R[(2, 2)].powf(2.) + R[(2, 0)].powf(2.)).sqrt();
    let s = R[(2, 0)] / (R[(2, 2)].powf(2.) + R[(2, 0)].powf(2.)).sqrt();
    let Qy = Matrix3::from_vec(vec![c, 0., -s, 0., 1., 0., s, 0., c]);
    let R = &R * &Qy;

    let c = -R[(1, 1)] / (R[(1, 1)].powf(2.) + R[(1, 0)].powf(2.)).sqrt();
    let s = R[(1, 0)] / (R[(1, 1)].powf(2.) + R[(1, 0)].powf(2.)).sqrt();
    let Qz = Matrix3::from_vec(vec![c, s, 0., -s, c, 0., 0., 0., 1.]);
    let K = &R * &Qz;
    let mut K = Matrix3::from_vec(K.data.as_vec().to_vec());
    let mut K = K / K[(2, 2)];
    let mut R = Qz.transpose() * Qy.transpose() * Qx.transpose();
    for y in 0..3 {
        for x in 0..3 {
            if K[(y, x)] < 0. {
                K[(y, x)] = -K[(y, x)];
                R[(y, x)] = -R[(y, x)];
            }
        }
    }
    let mut temp = Matrix3::from_element(0.);
    temp.column_mut(0).copy_from(&p.column(1));
    temp.column_mut(1).copy_from(&p.column(2));
    temp.column_mut(2).copy_from(&p.column(3));
    let x = temp.determinant();
    temp.column_mut(0).copy_from(&p.column(0));
    temp.column_mut(1).copy_from(&p.column(2));
    temp.column_mut(2).copy_from(&p.column(3));
    let y = -temp.determinant();
    temp.column_mut(0).copy_from(&p.column(0));
    temp.column_mut(1).copy_from(&p.column(1));
    temp.column_mut(2).copy_from(&p.column(3));
    let z = temp.determinant();
    temp.column_mut(0).copy_from(&p.column(0));
    temp.column_mut(1).copy_from(&p.column(1));
    temp.column_mut(2).copy_from(&p.column(2));
    let w = -temp.determinant();
    let c = Vector3::from_vec(vec![x / w, y / w, z / w]);
    let t = -1. * R * c;
    let pose = R;
    let mut pose = pose.insert_column(3, 0.);
    pose.column_mut(3).copy_from(&t);
    let pose = Matrix3x4::from_vec(pose.data.as_slice().to_vec());
    (K, pose)
}

// 本质矩阵的分解
// 获得R,T
pub fn essential_decomposition(
    essential: &Array2<f64>,
) -> (Matrix3<f64>, Matrix3<f64>, Vector3<f64>, Vector3<f64>) {
    let essential = essential.clone().to_owned().into_nalgebra();
    let mut svd = essential.svd(true, true);
    sort_svd(&mut svd);
    let W = Matrix3::from_vec(vec![0., -1., 0., 1., 0., 0., 0., 0., 1.]);
    let U = svd.u.unwrap();
    let Vt = svd.v_t.unwrap();
    // let Tx = U * Matrix3::from_vec(vec![1., 0., 0., 0., 1., 0., 0., 0., 0.]);
    let R1 = &U * W * &Vt;
    let R1 = R1.determinant() * R1;
    let R1 = Matrix3::from_vec(R1.data.as_vec().clone());

    let R2 = &U * W.transpose() * &Vt;
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
    let vec_poses = vec![(R1, T1), (R2, T1), (R1, T2), (R2, T2)];
    let counts: Vec<usize> = vec_poses
        .iter()
        .map(|(R, T)| {
            let matches = match_points_1.iter().zip(match_points_2);
            let world_ps: Vec<Option<Vector3<f64>>> = matches
                .map(|(p1, p2)| {
                    let p1 = Vector2::new(p1.x, p1.y);
                    let p2 = Vector2::new(p2.x, p2.y);
                    let triangulator = RelativeDltTriangulator::new();

                    let mut relative_pose = R.clone();
                    let mut relative_pose = relative_pose.insert_column(3, 0.);

                    relative_pose.column_mut(3).copy_from(T);

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
                    let p2 = R * p + T;
                    if p.z > 0. && p2.z > 0. {
                        acc + 1
                    } else {
                        acc
                    }
                }
            })
        })
        .collect();
    let (max_index, _) = counts
        .iter()
        .enumerate()
        .max_by(|x, y| x.1.cmp(y.1))
        .unwrap();
    // vec_poses[max_index].to_owned()
    let (R, T) = &vec_poses[max_index];
    // let relative_pose = R.clone().to_owned().insert_fixed_rows(3, T.to_owned());
    let relative_pose =
        IsometryMatrix3::from_parts(Translation::from(T.clone()), Rotation3::from_matrix(R))
            .to_matrix();

    relative_pose
        .ref_ndarray2()
        .to_owned()
        .slice(s![0..3, ..])
        .to_owned()
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
    use nshare::ToNalgebra;

    use crate::{
        point::Point2,
        sfm::{
            find_pose_by_essential, get_projection_through_fundamental, projection_decomposition,
        },
    };

    use super::{compute_projection_qr_decomposition, essential_decomposition};

    // use crate::sfm::{find_pose, };
    #[test]
    fn test_find_pose() {
        let shape = [3, 3];
        let fundamental = array![
            [2.79581382e-06, -7.78641696e-06, 5.09004126e-03],
            [1.21599362e-05, -8.33557647e-07, 3.61811421e-03],
            [-7.47814732e-03, -5.23955075e-03, 1.00000000e+00]
        ];
        let projection_matrix = get_projection_through_fundamental(&fundamental);

        // let projection_matrix = Matrix3x4::from_vec(vec![
        //     -7.48030043e-01,
        //     1.80474514e+0,
        //     1.76908427e-02,
        //     8.04740348e-01,
        //     -1.94187082e+00,
        //     8.07168914e-03,
        //     8.95374068e+01,
        //     -2.15996573e+02,
        //     -4.54385772e+00,
        //     2.16003609e+02,
        //     8.95192868e+01,1.
        // ]);
        println!("projection {:?}", projection_matrix);
        let (k_inner, pose) = compute_projection_qr_decomposition(&projection_matrix);
        println!("k_inner {:?} \n pose {:?} ", k_inner, pose);
    }

    #[test]
    fn test_projection_qr() {
        let p = Matrix3x4::from_vec(vec![
            3.53553e2,
            -1.03528e2,
            7.07107e-1,
            3.39645e2,
            2.33212e1,
            -3.53553e-1,
            2.77744e2,
            4.59607e2,
            6.12372e-1,
            -1.44946e6,
            -6.32525e5,
            -9.18559e2,
        ]);
        let (k, pose) = compute_projection_qr_decomposition(&p);
        println!("k {:?}", k);
        println!("pose{:?}", pose);
    }

    #[test]
    // 520.9, 0, 325.1, 0, 521.0, 249.7, 0, 0, 1
    // [4.544437503937326e-06, 0.0001333855576988952, -0.01798499246457619;
    // -0.0001275657012959839, 2.266794804637672e-05, -0.01416678429258694;
    // 0.01814994639952877, 0.004146055871509035, 1]
    // essential_matrix is
    // [0.01097677479889588, 0.2483720528328748, 0.03167429208264108;
    // -0.2088833206116968, 0.02908423961947315, -0.674465883831914;
    // 0.008286777626839029, 0.66140416240827, 0.01676523772760232]
    // homography_matrix is
    // [0.9261214281395963, -0.1445322024422802, 33.26921085290552;
    // 0.04535424466077615, 0.9386696693994352, 8.570979963061975;
    // -1.00619755759245e-05, -3.0081402779533e-05, 1]
    // R is
    // [0.9969387384756405, -0.0515557418857258, 0.05878058527448649;
    // 0.05000441581116598, 0.9983685317362444, 0.02756507279509838;
    // -0.06010582439317147, -0.02454140007064545, 0.9978902793176159]
    // t is
    // [-0.9350802885396324;
    // -0.03514646277098749;
    // 0.352689070059345]
    // t^R=
    // [-0.01552350379194682, -0.3512511256306389, -0.04479421344178829;
    // 0.2954056249626309, -0.04113132612112196, 0.9538388002732133;
    // -0.01171927330817152, -0.9353667366876339, -0.02370962657084997]
    fn test_esstinal() {
        let essential = array![
            [0.01097677479889588, 0.2483720528328748, 0.03167429208264108],
            [-0.2088833206116968, 0.02908423961947315, -0.674465883831914],
            [0.008286777626839029, 0.66140416240827, 0.01676523772760232],
        ];
        let result = essential_decomposition(&essential);
        println!("result {:?}", result);
        let p1s = [
            (194, 32),
            (162, 40),
            (151, 50),
            (208, 150),
            (154, 194),
            (194, 228),
            (92, 232),
            (403, 242),
            (186, 262),
            (339, 307),
            (348, 308),
            (401, 347),
        ];
        let p2s = [
            (226, 44),
            (193, 51),
            (183, 60),
            (208, 160),
            (142, 200),
            (180, 232),
            (90, 234),
            (376, 258),
            (170, 266),
            (236, 306),
            (311, 317),
            (357, 358),
        ];
        let p1s = p1s
            .iter()
            .map(|p| Point2::new(p.0 as f64, p.1 as f64))
            .collect();
        let p2s = p2s
            .iter()
            .map(|p| Point2::new(p.0 as f64, p.1 as f64))
            .collect();

        let pose = find_pose_by_essential(&essential, &p1s, &p2s);
        println!("pose {:?}", pose);
    }
}
