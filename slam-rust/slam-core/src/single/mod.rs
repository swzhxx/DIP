//! 单视图结构恢复

use crate::{point::Point2, svd::compute_min_vt_eigen_vector};

use nalgebra::Matrix3x4;
use ndarray::{array, Array2, Axis};
use nshare::ToNalgebra;

pub struct SingleViewRecover {}

impl SingleViewRecover {
    /// 给定2个二维空间的点，计算线性方程的截距表达式
    ///
    /// # Examples #
    ///
    /// ```ignore
    /// use slam_core::single::SingleViewRecover;
    /// use slam_core::point::Point2;
    /// let (slope , b) = SingleViewRecover::linear_equation( Point2::new(1.0,1.0),  Point2::new(0.,0.));
    /// assert_eq!(slope , 1.);
    /// assert_eq!(b , 0.);
    /// ```
    pub fn linear_equation(p1: &Point2<f64>, p2: &Point2<f64>) -> (f64, f64) {
        let x1 = p1.x;
        let y1 = p1.y;
        let x2 = p2.x;
        let y2 = p2.y;

        // 计算斜率slope
        let k = (y2 - y1) / (x2 - x1);
        let b = y1 - k * x1;
        (k, b)
    }

    /// 给定三维空间中2个平行线，并分别给每个平行线选取2个点。
    ///
    /// 求的像素空间中影消点
    pub fn find_vanshing_point(
        p1: &Point2<f64>,
        p2: &Point2<f64>,
        p3: &Point2<f64>,
        p4: &Point2<f64>,
    ) -> Point2<f64> {
        let (k1, b1) = SingleViewRecover::linear_equation(p1, p2);
        let (k2, b2) = SingleViewRecover::linear_equation(p3, p4);
        // 求焦点

        let x = (b2 - b1) / (k1 - k2);
        let y = k1 * x + b1;
        Point2::new(x, y)
    }

    /// 在像素空间中寻找3个不共面的影消点，通过影消点求的摄像机的内参数
    pub fn compute_camera_params_from_vanshing_points(
        vp1: &Point2<f64>,
        vp2: &Point2<f64>,
        vp3: &Point2<f64>,
    ) -> Array2<f64> {
        // let v1 = &vp1.data;
        // let v2 = &vp2.data;
        // let v3 = &vp3.data;

        let mut a: Array2<f64> = Array2::<f64>::from_elem((0, 4), 0.);

        a.push(
            Axis(0),
            array![
                vp1.x * vp2.x + vp1.y * vp2.y,
                vp1.x + vp2.x,
                vp2.x + vp2.y,
                1.
            ]
            .view(),
        )
        .unwrap();

        a.push(
            Axis(0),
            array![
                vp1.x * vp3.x + vp1.y * vp3.y,
                vp1.x + vp3.x,
                vp3.x + vp3.y,
                1.
            ]
            .view(),
        )
        .unwrap();

        a.push(
            Axis(0),
            array![
                vp2.x * vp3.x + vp2.y * vp3.y,
                vp2.x + vp3.x,
                vp3.x + vp3.y,
                1.
            ]
            .view(),
        )
        .unwrap();

        let na = a.view().into_nalgebra().into_owned();
        let w = compute_min_vt_eigen_vector(&na);
        // let svd = na.svd(true, true);
        // let vt = svd.v_t.unwrap();
        // let w = vt.column(3);
        // println!("w {:?}", w.len());
        // println!("w[0] {:?}", w[0]);
        // println!("w[1] {:?}", w[1]);
        // println!("w[2] {:?}", w[2]);
        // println!("w[3] {:?}", w[3]);
        let omega = array![[w[0], 0., w[1]], [0., w[0], w[2]], [w[1], w[2], w[3]]];

        let kt_inv = omega.into_nalgebra().cholesky().unwrap().l();
        let mut k = kt_inv.transpose().try_inverse().unwrap();
        // let k = DVector::<f64>::from_column_slice(data)
        let k: Vec<f64> = k.data.into();
        let k22 = *k.get(k.len() - 1).unwrap();
        let mut k = Array2::from_shape_vec((3, 3), k).unwrap();
        k * (1. / k22)
    }
}

#[cfg(test)]
mod test {
    use super::SingleViewRecover;
    use crate::point::Point2;
    #[test]
    fn compute_camera_params_from_vanshing_points() {
        let vp1: Point2<f64> = Point2::new(-1600.5, -378.5);
        let vp2: Point2<f64> = Point2::new(4852.5580716, -33.271318753352034);
        let vp3: Point2<f64> = Point2::new(2811.714285714286, -4572.571428571429);
        let result =
            SingleViewRecover::compute_camera_params_from_vanshing_points(&vp1, &vp2, &vp3);
    }
}
// (array([[ 0.48261984,  0.85470801, -0.19118659],
//   [ 0.17237483,  0.12132715,  0.97753089],
//   [-0.8586996 ,  0.50473156,  0.08877526]]),
// array([1.60663217e+07, 7.43446220e+03, 3.06244207e+03]),
// array([[-9.99999931e-01, -2.98949389e-04,  2.19988330e-04,
//    -1.26790027e-08],
//   [-1.83967610e-04,  9.13975103e-01,  4.05770178e-01,
//     1.99176036e-04],
//   [-3.22368589e-04,  4.05770049e-01, -9.13975099e-01,
//     2.85758731e-04],
//   [ 1.16082584e-07, -2.97994294e-04,  1.80356683e-04,
//     9.99999939e-01]]))
