use std::{convert::TryFrom, ops::Deref};

use ndarray::{Array1, Ix1};
use num_traits::{AsPrimitive, Num, ToPrimitive};

// pub type Point2<T> = ArrayBase<T, [usize; 2]>;
// pub type Point3<T> = ArrayBase<T, [usize; 3]>;

pub trait Point: PartialEq + PartialOrd + Copy + Clone + Num + ToPrimitive {}
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

    pub fn f(&self) -> Point2<f64> {
        Point2::new(self.x.to_f64().unwrap(), self.y.to_f64().unwrap())
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

impl<T> TryFrom<Array1<T>> for Point2<T>
where
    T: Point,
{
    type Error = &'static str;
    fn try_from(arr: Array1<T>) -> Result<Self, Self::Error> {
        if arr.len() < 3 {
            Err("Array1 len must >= 2")
        } else {
            Ok(Point2::new(arr[0], arr[1]))
        }
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
    pub fn f(&self) -> Point3<f64> {
        Point3::new(
            self.x.to_f64().unwrap(),
            self.y.to_f64().unwrap(),
            self.z.to_f64().unwrap(),
        )
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

impl<T> TryFrom<Array1<T>> for Point3<T>
where
    T: Point,
{
    type Error = &'static str;
    fn try_from(arr: Array1<T>) -> Result<Self, Self::Error> {
        if arr.len() < 3 {
            Err("Array1 len must >= 3")
        } else {
            Ok(Point3::new(arr[0], arr[1], arr[2]))
        }
    }
}
