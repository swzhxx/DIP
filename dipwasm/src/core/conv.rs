use crate::core::kernel::Kernel;
use ndarray::prelude::*;
use num_traits::{cast::ToPrimitive, Num};

pub fn conv_2d<T, D>(data: &mut Array3<T>, kernel: &Kernel<D>, padding_value: T) -> Array3<f64>
where
    T: Num + Copy + ToPrimitive,
{
    let shape = data.shape().to_vec();
    let padding = || {
        let new_shape: Vec<usize> = shape.iter().map(|val| *val + 2).collect();
        let new_shape = (new_shape[0], new_shape[1], new_shape[2]);
        let mut new_array: Array3<f64> =
            Array::from_shape_fn(new_shape, |(_)| padding_value.to_f64().unwrap());
        new_array
            .slice_mut(s![1..shape[0], 1..shape[1], ..])
            .assign(&data.slice(s![.., .., ..]).map(|val| val.to_f64().unwrap()));
        new_array
    };
    padding();
    unimplemented!()
}

// pub fn return_0_with_type<T>(input: T) -> T
// where
//     T: Default,
// {
//     T::default()
// }

// pub fn push_vec_with_0<T>(mut input: Vec<T>) -> Vec<T>
// where
//     T: Default,
// {
//     input.push(T::default());
//     input
// }

// #[cfg(test)]
// mod test {
//     use super::*;
//     #[test]
//     fn return_0_with_type_u8() {
//         let ret = return_0_with_type(0 as u8);
//         assert_eq!(ret, 0 as u8);
//     }
//     fn return_0_with_type_f64() {
//         let ret = return_0_with_type(0. as f64);
//         assert_eq!(ret, 0. as f64);
//     }

//     fn push_vec_with_0_u8() {
//         let v: Vec<u8> = vec![1, 2, 3];
//         assert_eq!(push_vec_with_0(v), [1, 2, 3, 0]);
//     }
//     fn push_vec_with_0_f64() {
//         let v: Vec<f64> = vec![1., 2., 3.];
//         assert_eq!(push_vec_with_0(v), [1., 2., 3., 0.]);
//     }
// }
