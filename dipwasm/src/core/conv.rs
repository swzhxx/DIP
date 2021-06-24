use std::convert::TryInto;
use std::ops::Mul;

use crate::core::kernel::Kernel;
use crate::core::Error;
use ndarray::{prelude::*, RawData, RawDataClone};
use ndarray::{Array3, DataMut};
use ndarray::{Data, DataShared};
use num_traits::NumAssignOps;
use num_traits::{cast::ToPrimitive, Num, Zero};

pub trait Convolution<T>
where
    T: Num + Copy + Clone + NumAssignOps + ToPrimitive + Default,
{
    type Output;
    fn conv_2d<U: Data + Data<Elem = T>>(&self, kernel: &Kernel<T>) -> Result<Self::Output, Error>;
}

impl<T> Convolution<T> for Array3<T>
where
    T: Num + Copy + Clone + NumAssignOps + ToPrimitive + Default,
{
    type Output = Array3<T>;

    fn conv_2d<U: Data + Data<Elem = T> + RawData>(
        &self,
        kernel: &Kernel<T>,
    ) -> Result<Self::Output, Error> {
        // Data prepare
        let shape = self.shape();
        let padding_shape = [shape[0] + 2, shape[1] + 2, shape[3]];
        // 填充0
        let mut padding_array = Array::from_shape_fn(padding_shape, |_| Zero::zero());
        padding_array
            .slice_mut(s![1..shape[0] - 1, 1..shape[1] - 1, ..])
            .assign(self);
        // Convolution start
        // let kernel_shape = kernel.shape();
        let raw_kernel = kernel.clone().into_shape([1, kernel.len()]).unwrap();
        let channels = shape[2];
        for y in 1..padding_shape[0] - 1 {
            for x in 1..padding_shape[1] - 1 {
                for channel in 0..channels {
                    let raw_data = padding_array
                        .slice_mut(s![y..y + shape[0], x..x + shape[1], channel])
                        .into_shape((1, kernel.len()))
                        .unwrap();
                    let conv = &raw_data * &raw_kernel;
                    padding_array[[y, x, channel]] = conv.sum();
                }
            }
        }
        Ok(padding_array)
    }
}

// pub fn conv_2d<T, D>(data: &mut Array3<T>, kernel: &Kernel<D>, padding_value: T) -> Array3<f64>
// where
//     T: Num + Copy + ToPrimitive,
// {
//     let shape = data.shape().to_vec();
//     let padding = || {
//         let new_shape: Vec<usize> = shape.iter().map(|val| *val + 2).collect();
//         let new_shape = (new_shape[0], new_shape[1], new_shape[2]);
//         let mut new_array: Array3<f64> =
//             Array::from_shape_fn(new_shape, |(_)| padding_value.to_f64().unwrap());
//         new_array
//             .slice_mut(s![1..shape[0], 1..shape[1], ..])
//             .assign(&data.slice(s![.., .., ..]).map(|val| val.to_f64().unwrap()));
//         new_array
//     };
//     padding();
//     unimplemented!()
// }

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
