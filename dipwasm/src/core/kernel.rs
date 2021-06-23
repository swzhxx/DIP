use ndarray::prelude::*;

pub fn kernel_centre(rows: usize, cols: usize) -> (usize, usize) {
    unimplemented!()
}

#[derive(Debug)]
pub struct Kernel<T> {
    data: Array2<T>,
}

impl<T> Kernel<T> {
    pub fn new(data: Array2<T>) -> Self {
        Kernel { data }
    }
}

/// create gaussian kernel
pub fn gaussian_kernel(size: usize) -> Kernel<f64> {
    let sigma: f64 = 0.3 * ((size / 2 - 1) as f64) + 0.8;
    let mut window = Array::from_shape_fn((size, size), |(i, j)| 0.);
    let gaussian = |y: usize, x: usize| {
        let t = y.pow(2) + x.pow(2);
        let sigma_square = sigma.powi(2);
        let res = (t as f64 / sigma_square).exp();
        return res;
    };
    let shape = window.shape().to_vec();
    for y in 0..shape[0] {
        for x in 0..shape[1] {
            let value = gaussian(y, x);
            window[[y, x]] = value;
        }
    }
    Kernel::new(window)
}
