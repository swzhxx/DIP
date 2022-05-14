use std::any::type_name;

use cv_3d::block_match::Ncc;
use num_traits::ToPrimitive;

fn test_type<T>(_: T) {
    println!("{:?}", { type_name::<T>() });
}

fn main() {
    let a = 2;
    println!("{}", a.to_f32().unwrap());
    // let ncc = Ncc::default();
    // test_type(ncc)
}
