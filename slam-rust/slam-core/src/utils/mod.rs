use ndarray::{array, s, Array, Array1, Array2};

use crate::point::{Point, Point2, Point3};

/// @param p 为像素坐标 ，
///
/// @param camera_inner_args 为摄像机内参数
pub fn px2cam<T>(p: Point2<T>, camera_inner_args: Array2<f64>) -> Point3<f64>
where
    T: Point,
{
    let cx = camera_inner_args[[0, 2]];
    let cy = camera_inner_args[[1, 2]];
    let fx = camera_inner_args[[0, 0]];
    let fy = camera_inner_args[[1, 1]];

    return Point3::new(
        (p.x.to_f64().unwrap() - cx) / fx,
        (p.y.to_f64().unwrap() - cy) / fy,
        1.,
    );
}

pub fn cam2px<T>(p: Point3<T>, camera_inner_args: Array2<f64>) -> Point2<f64>
where
    T: Point,
{
    let cx = camera_inner_args[[0, 2]];
    let cy = camera_inner_args[[1, 2]];
    let fx = camera_inner_args[[0, 0]];
    let fy = camera_inner_args[[1, 1]];
    return Point2::new(
        p.x.to_f64().unwrap() * fx / p.z.to_f64().unwrap() + cx,
        p.y.to_f64().unwrap() * fy / p.x.to_f64().unwrap() + cy,
    );
}

pub fn inside() -> bool {
    todo!()
}

/// ncc匹配
///
/// @param image1 参考图像
///
/// @param image2 当前图像
///
/// @param pt_ref 参考坐标点
///
/// @param pt_curr 当前坐标点
///
/// @param ncc_window_size 比较的窗口大小 @default 3
pub fn ncc(
    image1: &Array2<f64>,
    image2: &Array2<f64>,
    pt_ref: (usize, usize),
    px_curr: (f64, f64),
    ncc_window_size: Option<usize>,
) -> f64 {
    let ncc_window_size = match ncc_window_size {
        Some(val) => val,
        _ => 3,
    };
    let ncc_area = (2 * ncc_window_size + 1).pow(2);
    let ncc_window_size = ncc_window_size;
    let ref_block = image1.slice(s![
        pt_ref.0 - ncc_window_size..pt_ref.0 + ncc_window_size,
        pt_ref.1 - ncc_window_size..pt_ref.1 + ncc_window_size
    ]);
    let curr_block = image2.slice(s![
        px_curr.0 as usize - ncc_window_size..px_curr.0 as usize + ncc_window_size,
        px_curr.1 as usize - ncc_window_size..px_curr.1 as usize + ncc_window_size
    ]);
    let mean_ref = ref_block.mean().unwrap();
    let mean_ref_mat =
        Array::from_elem([ncc_window_size * 2 + 1, ncc_window_size * 2 + 1], mean_ref);
    let mean_curr = curr_block.mean().unwrap();
    let mean_curr_mat = Array::from_elem(
        [ncc_window_size * 2 + 1, ncc_window_size * 2 + 1],
        mean_curr,
    );
    let numberator = ((&ref_block - &mean_ref_mat) * (&curr_block - &mean_curr_mat)).sum();
    let divisior = ((&ref_block - &mean_ref_mat)
        .t()
        .dot(&(&ref_block - &mean_ref_mat))
        .sum()
        * &(&curr_block - &mean_curr_mat)
            .t()
            .dot(&(&curr_block - &mean_curr_mat))
            .sum())
        .sqrt();

    numberator / (divisior + 1e-18)
}
