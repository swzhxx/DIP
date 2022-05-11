use std::any::type_name;

use cv_3d::block_match::Ncc;

fn test_type<T>(_: T) {
    println!("{:?}", { type_name::<T>() });
}

fn main() {
    // let ncc = Ncc::default();
    // test_type(ncc)
}
