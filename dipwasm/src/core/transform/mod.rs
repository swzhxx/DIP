use ndarray::prelude::*;
use ndarray::Array;
use num_traits::*;
use std::f64::consts::PI;

pub fn dct2d(data: &Array2<f64>) -> Array2<f64> {
    let shape = data.shape();
    let v = shape[0];
    let u = shape[1];

    let C = Array::from_shape_fn((u, v), |(y, x)| {
        if y == 0 && x == 0 {
            return 1 / v;
        } else {
            return 1 / (2 * v);
        }
    });

    let dct_matrix = |n: usize| -> Array2<f64> {
        let M = Array::from_shape_fn((n, n), |(y, x)| {
            let y = y as f64;
            let x = x as f64;
            let n = n as f64;
            (PI * y * (2. * n + 1.) / (2. * n)).cos() * (PI * y * (2. * n + 1.) / (2. * n)).cos()
        });
        M
    };

    dct_matrix(v).dot(data).dot(&dct_matrix(v))
}

fn dct_transform_matrix() -> Array2<f64> {
    let size = 8.0;
    Array::from_shape_fn((size as usize, size as usize), |(j, i)| {
        if j == 0 {
            1. / size.sqrt()
        } else {
            let j = j as f64;
            let i = i as f64;
            (2. / size).sqrt() * (PI * (2. * i + 1.) * j / (2. * size)).cos()
        }
    })
}

pub fn dct_2d_by_64_blocks(data: &Array2<f64>) -> Array2<f64> {
    let size = 8.;
    let T = dct_transform_matrix();

    let shape = data.shape();
    //割块
    let h_blocks = (shape[0] as f64 / size).ceil() as usize;
    let w_blocks = (shape[1] as f64 / size).ceil() as usize;

    let mut res: Array2<f64> = Array::from_elem((shape[0], shape[1]), 0.);

    for h_count in 0..h_blocks {
        for w_count in 0..w_blocks {
            let start_h = h_count * 8;
            let start_w = w_count * 8;
            let s = data.slice(s![start_h..(start_h + 8), start_w..(start_w + 8)]);
            res.slice_mut(s![start_h..(start_h + 8), start_w..(start_w + 8)])
                .assign(&(&s.dot(&T)));
        }
    }
    res
}

pub fn inverse_dct_2d_by_64_blocks(data: &Array2<f64>) -> Array2<f64> {
    let size = 8.;
    let T = dct_transform_matrix();
    let T = T.t();

    let shape = data.shape();
    //割块
    let h_blocks = (shape[0] as f64 / size).ceil() as usize;
    let w_blocks = (shape[1] as f64 / size).ceil() as usize;

    let mut res: Array2<f64> = Array::from_elem((shape[0], shape[1]), 0.);

    for h_count in 0..h_blocks {
        for w_count in 0..w_blocks {
            let start_h = h_count * 8;
            let start_w = w_count * 8;
            let s = data.slice(s![start_h..(start_h + 8), start_w..(start_w + 8)]);
            res.slice_mut(s![start_h..(start_h + 8), start_w..(start_w + 8)])
                .assign(&(&s.dot(&T)));
        }
    }
    res
}

#[cfg(test)]
mod test {
    use super::*;
    use ndarray::prelude::*;
    #[test]
    fn dct_8_transform_matrix() {
        let T = dct_transform_matrix();
        let T1 = array![
            [0.35355, 0.35355, 0.35355, 0.35355, 0.35355, 0.35355, 0.35355, 0.35355],
            [0.49039, 0.41573, 0.27779, 0.09755, -0.09755, -0.27779, -0.41573, -0.49039],
            [0.46194, 0.19134, -0.19134, -0.46194, -0.46194, -0.19134, 0.19134, 0.46194],
            [0.41573, -0.09755, -0.49039, -0.27779, 0.27779, 0.49039, 0.09755, -0.41573],
            [0.35355, -0.35355, -0.35355, 0.35355, 0.35355, -0.35355, -0.35355, 0.35355],
            [0.27779, -0.49039, 0.09755, 0.41573, -0.41573, -0.09755, 0.49039, -0.27779],
            [0.19134, -0.46194, 0.46194, -0.19134, -0.19134, 0.46194, -0.46194, 0.19134],
            [0.09755, -0.27779, 0.41573, -0.49039, 0.49039, -0.41573, 0.27779, -0.09755]
        ];
        let abs = &T - &T1;
        assert!(abs.sum() < 0.01);
    }
    #[test]
    fn test_dct_2d_by_64_blocks() {
        let data = Array2::from_shape_fn((8, 8), |(j, i)| if j == i { 1. } else { 0. });
        let dct_res = dct_2d_by_64_blocks(&data);

        let dct_transform_matrix = dct_transform_matrix();
        let dct_res2 = dct_transform_matrix.dot(&data);

        let abs = &dct_res - &dct_res2;
        assert_eq!(dct_res, dct_res2);
        // assert!(abs.sum() < 0.01)
    }
}
