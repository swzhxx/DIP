use std::collections::HashMap;

// use nalgebra::{geometry::Point2, Matrix2};
use crate::point::Point2;
use ndarray::Array2;
use num_traits::{abs, Num, Signed};

#[derive(Debug)]
/// OFAST算法寻找特征点
pub struct OFast<'a, T>
where
    T: PartialEq + PartialOrd + Num + Copy + Signed,
{
    pub data: &'a Array2<T>,
    pub radius: usize,
}
impl<T> OFast<'_, T>
where
    T: PartialEq + PartialOrd + Num + Copy + Signed,
{
    pub fn new<'a>(data: &'a Array2<T>) -> OFast<'a, T> {
        OFast { data, radius: 4 }
    }

    pub fn new_with_radius<'a>(data: &'a Array2<T>, radius: usize) -> OFast<'a, T> {
        OFast { data, radius }
    }

    pub fn find_features(&self) -> Vec<Point2<usize>> {
        let radius = self.radius;
        let shape = self.data.shape();
        let height = shape[0];
        let width = shape[1];
        let mut features: Vec<Point2<usize>> = vec![];

        for row_i in 0..height {
            if row_i < radius || row_i > height - radius - 1 {
                continue;
            }

            for col_i in 0..width {
                if col_i < radius || col_i > width - radius - 1 {
                    continue;
                }

                let value = self.data.get((row_i, col_i)).unwrap();
                let value_12 = self.data.get((row_i - radius, col_i)).unwrap();
                let value_3 = self.data.get((row_i, col_i + radius)).unwrap();
                let value_6 = self.data.get((row_i + radius, col_i)).unwrap();
                let value_9 = self.data.get((row_i, col_i - radius)).unwrap();
                if (value > value_12 && value > value_3 && value > value_6 && value > value_9)
                    || (value < value_12 && value < value_3 && value < value_6 && value < value_9)
                {
                    // 非极大值抑止
                    if let Some(distance) = self.compute_distance(col_i, row_i) {
                        let left_distance =
                            self.compute_distance(col_i - 1, row_i).unwrap_or(T::zero());
                        let top_distance =
                            self.compute_distance(col_i, row_i - 1).unwrap_or(T::zero());
                        let right_distance =
                            self.compute_distance(col_i + 1, row_i).unwrap_or(T::zero());
                        let bottom_distance =
                            self.compute_distance(col_i, row_i + 1).unwrap_or(T::zero());
                        if distance > left_distance
                            && distance > top_distance
                            && distance > right_distance
                            && distance > bottom_distance
                        {
                            features.push(Point2::new(col_i, row_i));
                        }
                    }
                }
            }
        }
        //TODO : 添加缩放不变形，图像金字塔

        features
        // features
    }

    fn compute_distance(&self, col_i: usize, row_i: usize) -> Option<T> {
        let value = *self.data.get((row_i, col_i)).unwrap();
        let value_12 = *self.data.get((row_i - 1, col_i)).unwrap_or(&T::zero());
        let value_3 = *self.data.get((row_i, col_i + 1)).unwrap_or(&T::zero());
        let value_6 = *self.data.get((row_i + 1, col_i)).unwrap_or(&T::zero());
        let value_9 = *self.data.get((row_i, col_i - 1)).unwrap_or(&T::zero());

        if (value > value_12 && value > value_3 && value > value_6 && value > value_9)
            || (value < value_12 && value < value_3 && value < value_6 && value < value_9)
        {
            let distance =
                (value - value_12) + (value - value_3) + (value - value_6) + (value - value_9);
            let distance = abs(distance);
            Some(distance)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use ndarray::Array2;

    use crate::{features::fast::OFast, point::Point2};
    #[test]
    fn test_find_features() {
        let data = vec![
            1, 2, 3, 4, 4, 5, 6, 7, 8, 5, 10, 11, 12, 13, 11, 7, 6, 5, 4, 2, 3, 2, 1, 0, 1,
        ];
        let matrix = Array2::from_shape_vec([5, 5], data).unwrap();
        let o_fast = OFast::new_with_radius(&matrix, 2);
        let features = o_fast.find_features();
        assert_eq!(features.len(), 1);
        assert_eq!(features[0], Point2::new(2, 2));
    }
}
