//! 单视图结构恢复

use crate::point::Point2;

use ndarray::{array, Array2, Axis};
use nshare::ToNalgebra;
use nshare::ToNdarray2;

pub struct SingleViewRecover {}

impl SingleViewRecover {
    /// 给定2个二维空间的点，计算线性方程的截距表达式
    ///
    /// # Examples #
    ///
    /// ```
    /// let (slope , b) = SingleViewRecover::linear_equation(p1: Point2::new(1.0,1.0), p2: Point2::new(0.,0.))
    /// assert(slope , 1.)
    /// assert(b , 0)
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

        let mut a: Array2<f64> = Array2::<f64>::from_elem((3, 4), 0.);

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

        let svd = a.into_nalgebra().svd(false, true);
        let vt = svd.v_t.unwrap();
        let w = vt.column(2);
        let omega = array![[w[0], 0., w[1]], [0., w[0], w[2]], [w[1], w[2], w[3]]];

        let kt_inv = omega.into_nalgebra().cholesky().unwrap().l();
        let mut k = kt_inv.transpose().try_inverse().unwrap();
        // let k = DVector::<f64>::from_column_slice(data)
        let k: Vec<f64> = k.data.into();
        let k = Array2::from_shape_vec((3, 3), k).unwrap();
        k
    }
}
