use std::collections::HashMap;

// use nalgebra::{geometry::Point2, Matrix2};
use crate::point::Point2;
use ndarray::Array2;
use num_traits::{abs, Num, Signed};

#[derive(Debug)]
/// OFAST算法寻找特征点
pub struct OFast<'a, T>
where
    T: PartialEq + PartialOrd + Num + Copy + Signed + Into<f64>,
{
    pub data: &'a Array2<T>,
}
impl<T> OFast<'_, T>
where
    T: PartialEq + PartialOrd + Num + Copy + Signed + Into<f64>,
{
    pub fn new<'a>(data: &'a Array2<T>) -> OFast<'a, T> {
        OFast { data }
    }

    // pub fn new_with_radius<'a>(data: &'a Array2<T>, radius: usize) -> OFast<'a, T> {
    //     OFast { data, radius }
    // }
    /// 寻找特征点
    ///
    /// threshold 为门限，超过周围的门限数据，才会被统计，default 10.
    ///
    pub fn find_features(&self, threshold: Option<f64>) -> Vec<Point2<usize>> {
        let t = {
            if let Some(thres) = threshold {
                thres
            } else {
                10.
            }
        };

        let shape = self.data.shape();
        let height = shape[0];
        let width = shape[1];
        let mut features: Vec<Point2<usize>> = vec![];

        for row_i in 0..height {
            if row_i < 4 || row_i > height - 4 - 1 {
                continue;
            }

            for col_i in 0..width {
                if col_i < 4 || col_i > width - 4 - 1 {
                    continue;
                }

                if self.is_feature(row_i, col_i, t) {
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

    fn is_feature(&self, row_i: usize, col_i: usize, threshold: f64) -> bool {
        // let radius = self.radius;
        let value = self.data.get((row_i, col_i)).unwrap();
        let value_1 = self.data.get((row_i - 3, col_i)).unwrap();
        let value_2 = self.data.get((row_i - 3, col_i + 1)).unwrap();
        let value_3 = self.data.get((row_i - 2, col_i + 2)).unwrap();
        let value_4 = self.data.get((row_i - 1, col_i + 3)).unwrap();
        let value_5 = self.data.get((row_i, col_i + 3)).unwrap();
        let value_6 = self.data.get((row_i + 1, col_i + 3)).unwrap();
        let value_7 = self.data.get((row_i + 2, col_i + 2)).unwrap();
        let value_8 = self.data.get((row_i + 3, col_i + 1)).unwrap();
        let value_9 = self.data.get((row_i + 3, col_i)).unwrap();

        let value_10 = self.data.get((row_i + 3, col_i - 1)).unwrap();
        let value_11 = self.data.get((row_i + 2, col_i - 2)).unwrap();
        let value_12 = self.data.get((row_i + 1, col_i - 3)).unwrap();
        let value_13 = self.data.get((row_i, col_i - 3)).unwrap();
        let value_14 = self.data.get((row_i + 1, col_i - 3)).unwrap();
        let value_15 = self.data.get((row_i + 2, col_i - 2)).unwrap();
        let value_16 = self.data.get((row_i + 3, col_i - 1)).unwrap();

        let values = vec![
            value_1, value_2, value_3, value_4, value_5, value_6, value_7, value_8, value_9,
            value_10, value_11, value_12, value_13, value_14, value_15, value_16,
        ];
        let mut op = 0;
        let mut continues = 0;
        let mut max_continues = continues;
        for compare_value in values {
            let current_op: i32 = {
                if value > compare_value {
                    1
                } else if value < compare_value {
                    -1
                } else {
                    0
                }
            };

            let delta: f64 = (*value - *compare_value).abs().into();
            if delta < threshold || current_op != op {
                max_continues = {
                    if max_continues > continues {
                        max_continues
                    } else {
                        continues
                    }
                };
                continues = 1
            } else {
                continues = continues + 1;
                max_continues = {
                    if max_continues > continues {
                        max_continues
                    } else {
                        continues
                    }
                };
            }
            op = current_op;
        }

        if max_continues >= 12 {
            true
        } else {
            false
        }
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
        let o_fast = OFast::new(&matrix);
        let features = o_fast.find_features(Some(1.));
        assert_eq!(features.len(), 1);
        assert_eq!(features[0], Point2::new(2, 2));
    }
}
