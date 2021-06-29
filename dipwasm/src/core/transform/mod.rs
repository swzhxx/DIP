use num_traits::*;
use std::f64::consts::PI;

use ndarray::prelude::*;
pub fn dct2d(data: Array2<f64>) -> Array2<f64> {
    let shape = data.shape();
    let v = shape[0];
    let u = shape[1];
    let calc_c = |y: usize, x: usize| -> f64 {
        todo!();
    };
    let dct = |data: ArrayView<f64, Ix1>, c: f64| -> Array1<f64> {
        let data = &data
            * &Array::linspace(0., data.len() as f64, data.len())
                .map(|&val| return ((PI * c * (2. * val + 1.)) / (2. * v as f64)).cos());
        data
    };

    // 如果按照公式那么将是O^4的算法。需要优化
    // for y in 0..v {
    //     for x in 0..u {
    //         let c = calc_c(y, x);
    //         let v = data[[y, x]];
    //         let temp =
    //     }
    // }

    array![[]]
}
