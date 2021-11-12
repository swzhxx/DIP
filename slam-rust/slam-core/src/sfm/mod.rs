mod eight_point;
use std::rc::Rc;

pub use eight_point::*;
use ndarray::{array, Array1, Array2};
use nshare::{ToNalgebra, ToNdarray2};

use crate::{
    optimize::LM,
    point::{Point, Point2, Point3},
    svd::compute_min_vt_eigen_vector,
};

/// 通过Fundamental矩阵计算旋转和平移矩阵
/// 获得一个旋转矩阵，和一个平移向量的反对称矩阵
pub fn find_pose(fundamental: &Array2<f64>) -> (Array2<f64>, Array2<f64>) {
    let f = fundamental.clone().to_owned().into_nalgebra();
    // let svd = f.svd(false, true);
    // // let v_t = svd.v_t.unwrap();
    // let b = svd.v_t.unwrap();
    let less_eigen_vector = compute_min_vt_eigen_vector(&f);
    let b = array![
        [0., -less_eigen_vector[2], less_eigen_vector[1]],
        [less_eigen_vector[2], 0., -less_eigen_vector[0]],
        [-less_eigen_vector[1], less_eigen_vector[0], 0.]
    ];
    let a = -&b.dot(fundamental);
    (a, b)
}

type MatchPoints<T> = Vec<Point2<T>>;

pub fn restoration_perspective_structure<T>(
    fundamental: &Array2<f64>,
    match1: &MatchPoints<T>,
    match2: &MatchPoints<T>,
    iter: Option<usize>,
) -> Array2<f64>
where
    T: Point,
{
    let (mut a, b) = find_pose(fundamental);
    let b = array![b[[2, 1]], b[[0, 2]], b[[1, 0]]];
    // let m = b.dot(&a);
    // let m = m.into_shape((m.len())).unwrap();
    a.push_column(b.view());
    let m = a;
    let m_len = m.len();
    let m = match m.into_shape(m_len) {
        Ok(val) => val,
        Err(e) => {
            panic!("{:?}", e);
        }
    };

    let match1: Vec<Point3<f64>> = match1.iter().map(|p| p.f().homogeneous()).collect();
    let match2: Vec<Point3<f64>> = match2.iter().map(|p| p.f().homogeneous()).collect();
    let mut lm = LM::new(
        &match2,
        &match1,
        Rc::new(Box::new(|args, _, input, output| {
            let x = args[0] * input.x + args[1] * input.y + args[2] * input.z + args[3];
            let y = args[4] * input.x + args[5] * input.y + args[6] * input.z + args[7];
            let z = args[8] * input.x + args[9] * input.y + args[10] * input.z + args[11];

            ((output.x - x).powf(2.) + (output.y - y).powf(2.) + (output.z - z).powf(2.)).sqrt()
        })),
        Rc::new(Box::new(|args, _, input, error| -> Array1<f64> {
            let x = args[0] * input.x + args[1] * input.y + args[2] * input.z + args[3];
            let y = args[4] * input.x + args[5] * input.y + args[6] * input.z + args[7];
            let z = args[8] * input.x + args[9] * input.y + args[10] * input.z + args[11];

            let mut jaco = Array1::from_elem((args.len()), 0.);
            jaco[0] = (1. / error.sqrt() + f64::MIN.abs()) * 2. * x * input[0];
            jaco[1] = (1. / error.sqrt() + f64::MIN.abs()) * 2. * x * input[1];
            jaco[2] = (1. / error.sqrt() + f64::MIN.abs()) * 2. * x * input[2];
            jaco[3] = (1. / error.sqrt() + f64::MIN.abs()) * 2. * x;

            jaco[4] = (1. / error.sqrt() + f64::MIN.abs()) * 2. * y * input[0];
            jaco[5] = (1. / error.sqrt() + f64::MIN.abs()) * 2. * y * input[1];
            jaco[6] = (1. / error.sqrt() + f64::MIN.abs()) * 2. * y * input[2];
            jaco[7] = (1. / error.sqrt() + f64::MIN.abs()) * 2. * y;

            jaco[8] = (1. / error.sqrt() + f64::MIN.abs()) * 2. * z * input[0];
            jaco[8] = (1. / error.sqrt() + f64::MIN.abs()) * 2. * z * input[1];
            jaco[10] = (1. / error.sqrt() + f64::MIN.abs()) * 2. * z * input[2];
            jaco[11] = (1. / error.sqrt() + f64::MIN.abs()) * 2. * z;
            jaco
        })),
        None,
    );
    let t;

    if let Some(iter_count) = iter {
        t = lm.optimize(&m, Some(iter_count), None);
    } else {
        t = lm.optimize(&m, Some(100), None);
    }

    t.into_shape((3, 4)).expect("t reshape failed")
}

#[cfg(test)]
mod test {
    use ndarray::{array, Array2};

    use crate::sfm::{find_pose, restoration_perspective_structure};
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

        println!("pose {:?} ", find_pose(&fundamental));
    }
}
