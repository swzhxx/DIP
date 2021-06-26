use ndarray::prelude::*;
use num_traits::Num;
use std::f64::consts::PI;
use std::ops::Deref;

pub fn kernel_centre(rows: usize, cols: usize) -> (usize, usize) {
    unimplemented!()
}

#[derive(Debug)]
pub struct Kernel<T>
where
    T: Num + Copy + Clone,
{
    data: Array2<T>,
}

// impl<T> Copy for Kernel<T> where T: Num + Copy + Clone {}
impl<T> Kernel<T>
where
    T: Num + Copy + Clone,
{
    pub fn clone(&self) -> Array2<T> {
        self.data.clone()
    }
    pub fn new(data: Array2<T>) -> Self {
        Kernel { data }
    }
}

impl<T> Deref for Kernel<T>
where
    T: Num + Copy + Clone,
{
    type Target = Array2<T>;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

/// create gaussian kernel
pub fn gaussian_kernel(size: usize) -> Kernel<f64> {
    let sigma: f64 = 0.3 * ((size / 2 - 1) as f64) + 0.8;
    let centre = size / 2;
    let mut window = Array::from_shape_fn((size, size), |(i, j)| 0.);
    let gaussian = |y: i64, x: i64| {
        let t: i64 = (y.pow(2) + x.pow(2));
        let t = t as f64;
        let sigma_square = sigma.powi(2);
        let res = 1. / (PI * 2. * sigma_square) * (-t / (2.0 * sigma_square)).exp();
        return res;
    };

    for y in 0..size {
        for x in 0..size {
            let value = gaussian((y as i64 - centre as i64), (x as i64 - centre as i64));
            window[[y, x]] = value;
        }
    }

    Kernel::new(window)
}
