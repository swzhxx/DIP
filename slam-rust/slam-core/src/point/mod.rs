use std::ops::Deref;

use ndarray::{Array1, Ix1};

// pub type Point2<T> = ArrayBase<T, [usize; 2]>;
// pub type Point3<T> = ArrayBase<T, [usize; 3]>;

#[derive(Debug, Clone)]
pub struct Point2<T>
where
    T: PartialEq + PartialOrd + Copy + Clone,
{
    data: Array1<T>,
    x: T,
    y: T,
}

impl<T> Point2<T>
where
    T: PartialEq + PartialOrd + Copy + Clone,
{
    pub fn new(x: T, y: T) -> Point2<T> {
        let array = Array1::from_vec(vec![x, y]);
        Point2 { data: array, y, x }
    }
}

impl<T> Deref for Point2<T>
where
    T: PartialOrd + PartialEq + Copy + Clone,
{
    type Target = Array1<T>;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> PartialEq for Point2<T>
where
    T: PartialOrd + PartialEq + Copy + Clone,
{
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}
#[derive(Debug, Clone)]
pub struct Point3<T>
where
    T: PartialEq + PartialOrd + Copy + Clone,
{
    data: Array1<T>,
    x: T,
    y: T,
    z: T,
}

impl<T> Point3<T>
where
    T: PartialEq + PartialOrd + Copy + Clone,
{
    pub fn new(x: T, y: T, z: T) -> Point3<T> {
        let array = Array1::from_vec(vec![x, y, z]);
        Point3 {
            data: array,
            y,
            x,
            z,
        }
    }
}

impl<T> Deref for Point3<T>
where
    T: PartialOrd + PartialEq + Copy + Clone,
{
    type Target = Array1<T>;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> PartialEq for Point3<T>
where
    T: PartialOrd + PartialEq + Copy + Clone,
{
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z
    }
}
