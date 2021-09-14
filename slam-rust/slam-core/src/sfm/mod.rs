mod eight_point;
use std::rc::Rc;

pub use eight_point::*;
use ndarray::{array, Array1, Array2};
use nshare::{ToNalgebra, ToNdarray2};

use crate::{
    optimize::LM,
    point::{Point, Point2, Point3},
};

/// 通过Fundamental矩阵计算旋转和平移矩阵
/// 获得一个旋转矩阵，和一个平移向量的反对称矩阵
pub fn find_pose(fundamental: Array2<f64>) -> (Array2<f64>, Array2<f64>) {
    let f = fundamental.view().into_nalgebra();
    let svd = f.svd(false, true);
    // let v_t = svd.v_t.unwrap();
    let b = svd.v_t.unwrap();
    let b = b.column(8).into_ndarray2().to_owned();
    let a = -&b.dot(&fundamental);
    (a, b)
}

type MatchPoints<T> = Vec<Point2<T>>;

pub fn restoration_perspective_structure<T>(
    fundamental: Array2<f64>,
    match1: &MatchPoints<T>,
    match2: &MatchPoints<T>,
    iter: Option<usize>,
) -> Array2<f64>
where
    T: Point,
{
    let (a, b) = find_pose(fundamental);
    let b = array![b[[2, 1]], b[[0, 2]], b[[1, 1]]];
    // let m = b.dot(&a);
    // let m = m.into_shape((m.len())).unwrap();
    let mut m = a.clone();
    let m_len = m.len();
    m.push_column(b.view()).unwrap();
    let m = m.into_shape(m_len).unwrap();
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

            jaco[4] = (1. / error.sqrt() + f64::MIN.abs()) * 2. * x * input[0];
            jaco[5] = (1. / error.sqrt() + f64::MIN.abs()) * 2. * x * input[1];
            jaco[6] = (1. / error.sqrt() + f64::MIN.abs()) * 2. * x * input[2];
            jaco[7] = (1. / error.sqrt() + f64::MIN.abs()) * 2. * x;

            jaco[8] = (1. / error.sqrt() + f64::MIN.abs()) * 2. * x * input[0];
            jaco[8] = (1. / error.sqrt() + f64::MIN.abs()) * 2. * x * input[1];
            jaco[10] = (1. / error.sqrt() + f64::MIN.abs()) * 2. * x * input[2];
            jaco[11] = (1. / error.sqrt() + f64::MIN.abs()) * 2. * x;
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

    t.into_shape((4, 3)).unwrap()
}
