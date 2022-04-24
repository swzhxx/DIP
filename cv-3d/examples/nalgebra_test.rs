use nalgebra::{Matrix3, Vector2};

fn main() {
    let a = Vector2::from_row_slice(&[1, 2]);
    println!("{:?}", a);
    let b = Vector2::from_column_slice(&[3, 4]).transpose();
    println!("{:?}", b);
    println!("{:?}", a * &b);
    let mut c = Matrix3::from_row_slice(&[1., 2., 3., 4., 5., 6., 7., 8., 9.]);
    println!("{:?}", c);
    let c = Matrix3::from_vec(vec![1., 2., 3., 4., 5., 6., 7., 8., 9.]);
    let svd = c.svd(true, true);
    let singular_values = svd.singular_values;
    println!("{:?}", singular_values);
}
