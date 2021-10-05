use std::fmt::Debug;

use num_traits::Num;

use crate::point::Point2;

pub mod orb;
pub trait Match: Num + PartialOrd + Copy + Debug {}

impl Match for usize {}

#[derive(Debug)]
pub struct DMatch<T>
where
    T: Match,
{
    pub i1: usize,
    pub i2: usize,
    pub distance: T,
}

pub fn hanming_distance(mut distance: usize) -> usize {
    let mut count = 0;
    loop {
        distance = distance & (distance - 1);
        if distance == 0 {
            break;
        }
        count = count + 1;
    }
    count
}
