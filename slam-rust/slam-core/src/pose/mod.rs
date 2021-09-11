mod eight_point;
pub use eight_point::*;
use ndarray::Array2;
use nshare::{ToNalgebra, ToNdarray2};

use crate::point::{Point, Point2};


/// 通过Fundamental矩阵计算旋转和平移矩阵
pub fn find_pose<T>(fundamental: Array2<f64>) -> (Array2<f64>, Array2<f64>)
where
    T: Point,
{
    let f = fundamental.view().into_nalgebra();
    let svd = f.svd(false, true);
    // let v_t = svd.v_t.unwrap();
    let b = svd.v_t.unwrap().column(8);

    let b = b.into_ndarray2().to_owned();
    let a = -&b.dot(&fundamental);
    (a, b)
}

pub fn find_post_with_lm(fundamental: Array2<f64>) -> (Array2<f64>, Array2<f64>) {
    
  todo!()
}
