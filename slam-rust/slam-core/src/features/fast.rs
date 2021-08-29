// use nalgebra::{geometry::Point2, Matrix2};
use crate::point::Point2;
use ndarray::Array2;

/// OFAST算法寻找特征点
pub struct OFast<'a, T>
where
    T: PartialEq + PartialOrd,
{
    data: &'a Array2<T>,
    radius: usize,
}
impl<T> OFast<'_, T>
where
    T: PartialEq + PartialOrd,
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
                if value > value_12 && value > value_3 && value > value_6 && value > value_9 {
                    // features.push(Point2::new((row_i, col_i)));
                    features.push(Point2::new(col_i, row_i));
                } else if value < value_12 && value < value_3 && value < value_6 && value < value_9
                {
                    features.push(Point2::new(col_i, row_i));
                }
            }
        }
        features
    }
}

#[cfg(test)]
mod tests {
    use ndarray::Array2;

    use crate::{features::fast::OFast, point::Point2};
    #[test]
    fn test_find_features() {
        let data = vec![
            1, 2, 3, 4, 4, 
            5, 6, 7, 8, 5, 
            10,11, 12, 13, 11,
            7, 6, 5, 4, 2, 
            3, 2, 1, 0, 1,
        ];
        let matrix = Array2::from_shape_vec([5, 5], data).unwrap();
        let o_fast = OFast::new_with_radius(&matrix, 2);
        let features = o_fast.find_features();
        assert_eq!(features.len(), 1);
        assert_eq!(features[0], Point2::new(2, 2));
    }
}
