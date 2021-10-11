use std::fmt::Debug;

use num_traits::Num;

use crate::point::Point2;

pub mod orb;
pub trait Match: Num + PartialOrd + Copy + Debug {}

impl Match for usize {}
impl Match for u32 {}

#[derive(Debug)]
pub struct DMatch<T>
where
    T: Match,
{
    pub i1: usize,
    pub i2: usize,
    pub distance: T,
}

pub fn hanming_distance(mut distance: u32) -> u32 {
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
        let v3 = 5;
        let v4 = 1;

        assert_eq!(1, hanming_distance(v3 ^ v4));

        let v1: [u32; 8] = [
            3252522806, 3528110876, 3399917974, 1948276593, 1480299088, 1046709034, 3517304299,
            3926182068,
        ]; 
        let v2: [u32; 8] = [
            4076450672, 2547484990, 2863932866, 1659686327, 1500312139, 837970906, 2016773369,
            734431349,
        ];
        let v3: [u32; 8] = [
            4034657136, 2526251326, 716707266, 551862707, 1361901643, 836905434, 2016904441,
            2043971685,
        ];
        let mut d1 = 0;
        let mut d2 = 0;
        for i in 0..8 {
            d1 = d1 + hanming_distance(v1[i] ^ v3[i]);
            d2 = d2 + hanming_distance(v2[i] ^ v3[i]);
        }

        assert!(d1 > d2)
    }
}
