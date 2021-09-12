mod eight_point;
use std::rc::Rc;

pub use eight_point::*;
use ndarray::Array2;
use nshare::{ToNalgebra, ToNdarray2};

use crate::{
    optimize::LM,
    point::{Point, Point2},
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
) -> Array2<f64>
where
    T: Point,
{
    let (a, b) = find_pose(fundamental);
    let m = b.dot(&a);
    let match1: MatchPoints<f64> = match1.iter().map(|p| p.f()).collect();
    let match2: MatchPoints<f64> = match2.iter().map(|p| p.f()).collect();
    LM::new(
        &match2,
        &match1,
        Rc::new(Box::new(|args, _, input, output| todo!())),
        Rc::new(Box::new(|args, _, input| todo!())),
        None,
    );
    todo!()
}
