use std::ops::Deref;

use ndarray::{Array1, Ix1};
use num_traits::{AsPrimitive, Num};

// pub type Point2<T> = ArrayBase<T, [usize; 2]>;
// pub type Point3<T> = ArrayBase<T, [usize; 3]>;

pub trait Point: PartialEq + PartialOrd + Copy + Clone + Num {}
impl Point for usize {}
impl Point for f64 {}
impl Point for u32 {}
impl Point for i32 {}
impl Point for u64 {}
#[derive(Debug, Clone)]
pub struct Point2<T>
where
    T: Point,
{
    pub data: Array1<T>,
    pub x: T,
    pub y: T,
}

impl<T> Point2<T>
where
    T: Point,
{
    pub fn new(x: T, y: T) -> Point2<T> {
        let array = Array1::from_vec(vec![x, y]);
        Point2 { data: array, y, x }
    }

    pub fn homogeneous(&self) -> Point3<T> {
        Point3::new(self.x, self.y, T::one())
    }
}

impl<T> Deref for Point2<T>
where
    T: Point,
{
    type Target = Array1<T>;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> PartialEq for Point2<T>
where
    T: Point,
{
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl<T> From<Point3<T>> for Point2<T>
where
    T: Point,
{
    fn from(item: Point3<T>) -> Point2<T> {
        Point2::new(item.x, item.y)
    }
}

#[derive(Debug, Clone)]
pub struct Point3<T>
where
    T: Point,
{
    pub data: Array1<T>,
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T> Point3<T>
where
    T: Point,
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
    T: Point,
{
    type Target = Array1<T>;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> PartialEq for Point3<T>
where
    T: Point,
{
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z
    }
}
