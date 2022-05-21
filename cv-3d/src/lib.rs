use cv::core::{MatExprTraitConst, MatTrait, MatTraitConst, ToInputArray};
use cv::imgproc::COLOR_BGR2GRAY;
use cv_convert::TryFromCv;
use nalgebra::{DMatrix, Vector2};
use opencv::core::{KeyPoint, Mat, UMatTraitConst, CV_8U};
use opencv::{self as cv};

pub mod block_match;
pub mod epipolar;
pub mod feature;
pub mod filter;
pub mod frame;
pub mod fundamental;
pub mod triangluate;
pub mod utils;
pub trait ToNaVector2 {
    fn to_vector2(&self) -> Vector2<f64>;
}

impl ToNaVector2 for KeyPoint {
    fn to_vector2(&self) -> Vector2<f64> {
        Vector2::new(self.pt.x as f64, self.pt.y as f64)
    }
}

pub trait U8C3ToNaMatrix: ToInputArray + MatTrait {
    fn to_na_matrix(&self) -> DMatrix<u8>;
}

impl U8C3ToNaMatrix for Mat {
    fn to_na_matrix(&self) -> DMatrix<u8> {
        let mat_size = self.mat_size();
        let mat = Mat::eye(mat_size[0], mat_size[1], CV_8U).unwrap();
        let mut mat = mat.to_mat().unwrap();
        cv::imgproc::cvt_color(&self, &mut mat, COLOR_BGR2GRAY, 0).unwrap();
        let na_mat = DMatrix::<u8>::try_from_cv(mat).unwrap();
        na_mat
    }
}

#[cfg(test)]
mod test {
    use crate::U8C3ToNaMatrix;
    use opencv::core::{Mat, MatExprTraitConst, MatTraitConst, CV_8UC3};
    #[test]
    fn u8c3_to_namatrix() {
        let mat = Mat::zeros(3, 3, CV_8UC3).unwrap().to_mat().unwrap();
        println!("a {:?}", mat.mat_size());
        let na_m = (mat).to_na_matrix();
        println!("{:?}", na_m);
    }
}
