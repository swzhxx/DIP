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
        if distance == 0 {
            break;
        }
        count = count + 1;
        distance = distance & (distance - 1);
    }
    count
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_hanming_distance() {
        let v3 = 5usize;
        let v4 = 1usize;

        assert_eq!(1, hanming_distance(v3 ^ v4));
    }
}
