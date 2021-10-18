use nalgebra::vector;
use ndarray::{array, Array, Array1, Array2};

use crate::utils::px2cam;
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
                    || self.depth_cov2_matrix > self.max_depth
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
    ) -> (Array1<f64>, Array1<f64>) {
        let camera = array![[1., 0., 1.], [0., 1., 1.], [0., 0., 1.]];
        let pt_world = px2cam(&vector![pt_ref[1], pt_ref[0]], &camera);
        let pt_world = vector![pt_world.x, pt_world.y, pt_world.z];
        let pt_world = pt_world.normalize() * depth;

        todo!()
    }
}

pub type ReaderResult = (Option<Array2<f64>>, Option<Array2<f64>>);