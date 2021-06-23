use ndarray::{Array3, Dim, Dimension, Ix3};
use ndarray_vision::processing::conv;
use ndarray_vision::processing::{
    kernels::{GaussianFilter, KernelBuilder},
    Error,
};
use wasm_bindgen::prelude::*;

fn make_gaussian_kernel(kernel_size: usize, channels: usize) -> Result<Array3<T>, Error> {
    let shape: Ix3 = Ix3(kernel_size, kernel_size, channels);
    GaussianFilter::build(shape)?
}
