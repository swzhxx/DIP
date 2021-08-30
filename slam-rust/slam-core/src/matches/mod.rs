use num_traits::Num;

use crate::point::Point2;

pub mod orb;

#[derive(Debug)]
pub struct DMatch<T>
where
    T: Num + PartialOrd + PartialEq + Copy,
{
    i1: usize,
    i2: usize,
    distance: T,
}

pub fn hanming_distance(mut distance: usize) -> usize {
    let mut count = 0;
    loop {
        count = count + 1;
        distance = distance & (distance - 1);
        if distance == 0 {
            break;
        }
    }
    count
}
