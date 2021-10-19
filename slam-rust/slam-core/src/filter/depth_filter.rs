use nalgebra::{vector, Const, Dynamic, Vector2};
use ndarray::{array, Array, Array1, Array2};
use nshare::ToNalgebra;

use crate::utils::{cam2px, ncc, px2cam};
const border: usize = 20;
struct DepthFilter {
    height: usize,
    width: usize,
    depth_matrix: Array2<f64>,
    depth_cov2_matrix: Array2<f64>,
    min_depth: f64,
    max_depth: f64,
    reader: Box<dyn Fn() -> ReaderResult>,
    ref_image: Option<Array2<f64>>,
    current_image: Option<Array2<f64>>, // images: Vec<Array2<f64>>,
}

impl DepthFilter {
    pub fn new(
        height: usize,
        width: usize,
        depth_mean: Option<f64>,
        depth_cov: Option<f64>,
        min_depth: Option<f64>,
        max_depth: Option<f64>,
        reader: Box<dyn Fn() -> ReaderResult>,
    ) -> Self {
        let depth_mean = match depth_mean {
            Some(val) => val,
            _ => 3.,
        };
        let depth_cov = match depth_cov {
            Some(val) => val,
            _ => 3.,
        };
        let min_depth = match min_depth {
            Some(val) => val,
            _ => 0.1,
        };
        let max_depth = match max_depth {
            Some(val) => val,
            _ => 10.,
        };
        DepthFilter {
            height,
            width,
            depth_matrix: Array::from_elem((height, width), depth_mean),
            depth_cov2_matrix: Array::from_elem((height, width), depth_cov),
            min_depth,
            max_depth,
            reader, // images,
            ref_image: None,
            current_image: None,
        }
    }

    fn reader(&self) -> ReaderResult {
        let reader = &self.reader;
        reader()
    }

    pub fn excute(&mut self) {
        let (option_image, option_pose) = self.reader();
        match &self.ref_image {
            Some(_ref_image) => self.current_image = option_image,
            None => self.ref_image = option_image,
        };
        if self.ref_image != None && self.current_image != None && option_pose != None {
            self.update(&option_pose.unwrap())
        }
    }

    /// 对整个深度图更新
    pub fn update(
        &mut self,
        // ref_image: &Array2<f64>,
        // current_image: &Array2<f64>,
        pose: &Array2<f64>,
    ) {
        for y in border..(self.height - border) {
            for x in border..(self.width - border) {
                if self.depth_cov2_matrix[[y, x]] < self.min_depth
                    || self.depth_cov2_matrix[[y, x]] > self.max_depth
                {
                    continue;
                }
                let pt_ref = array![x as f64, y as f64];
                let depth = self.depth_matrix[[y, x]];
                let depth_cov = self.depth_cov2_matrix[[y, x]];
                self.epipolar_search(&pt_ref, pose, depth, depth_cov);
            }
        }
    }

    /// 极线搜索
    /// @return (匹配点坐标， 极线方向)
    fn epipolar_search(
        &self,
        pt_ref: &Array1<f64>,
        pose: &Array2<f64>,
        depth: f64,
        depth_cov: f64,
    ) -> Option<Vector2<f64>> {
        let camera = array![[1., 0., 1.], [0., 1., 1.], [0., 0., 1.]];
        let pose = pose
            .clone()
            .into_nalgebra()
            .reshape_generic(Const::<3>, Dynamic { value: 3 });
        let pt_world = px2cam(&vector![pt_ref[1], pt_ref[0]], &camera);
        let pt_world = vector![pt_world.x, pt_world.y, pt_world.z];
        let pt_world = pt_world.normalize() * depth;

        let px_mean_curr = cam2px(&(pose * &pt_world), &camera);
        let mut d_min = depth - 3. * depth_cov;
        let d_max = depth + 3. * depth_cov;
        if d_min < 0.1 {
            d_min = 0.1
        }
        let px_min_curr = cam2px(&(pt_world * d_min), &camera);
        let px_max_curr = cam2px(&(pt_world * d_max), &camera);
        let epipolar_line = px_max_curr - px_min_curr;
        let epipolar_direction = epipolar_line.normalize();
        let mut half_length = 0.5 * epipolar_line.norm();
        if half_length > 100. {
            half_length = 100.;
        }
        let best_ncc = -1.;
        let mut best_px_curr;
        let l = -half_length;
        while l <= half_length {
            let px_curr = px_mean_curr + l * epipolar_direction;
            if !inside(&px_curr, self.width, self.height) {
                continue;
            }
            let ncc_value = ncc(
                &self.ref_image.unwrap(),
                &self.current_image.unwrap(),
                (pt_ref[0], pt_ref[1]),
                (px_curr.x, px_curr.y),
                Some(3),
            );
            if ncc_value > best_ncc {
                best_ncc = ncc_value;
                best_px_curr = px_curr;
            }
            l = l + 0.7;
        }
        if best_ncc < 0.85 {
            None
        } else {
            Some(best_px_curr)
        }
    }
}

pub type ReaderResult = (Option<Array2<f64>>, Option<Array2<f64>>);

fn inside(pt: &Vector2<f64>, width: usize, height: usize) -> bool {
    let boarder: f64 = 20.;
    return pt[(0, 0)] >= boarder
        && pt[(1, 0)] >= boarder
        && pt[(0, 0)] + boarder < width as f64
        && pt[(1, 0)] + boarder <= height as f64;
}
