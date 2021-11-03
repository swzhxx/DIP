use std::{f64::consts::PI, marker::PhantomData, thread::current};

use nalgebra::{
    matrix, vector, AbstractRotation, Const, DMatrix, Dynamic, Matrix2, Matrix3, MatrixXx3,
    Vector2, Vector3,
};
use ndarray::{array, s, Array, Array1, Array2, ArrayBase, Axis, OwnedRepr};
use nshare::ToNalgebra;

use crate::utils::{cam2px, ncc, px2cam};
const border: usize = 20;

pub struct DepthFilter<'a> {
    images: &'a Vec<Array2<f64>>,
    height: usize,
    width: usize,
    pub depth_matrix: Array2<f64>,
    depth_cov2_matrix: Array2<f64>,
    min_depth: f64,
    max_depth: f64,
    reader: Box<dyn Fn(&'a Vec<Array2<f64>>) -> ReaderResult<'a>>,
    // ref_image: Option<&'a Array2<f64>>,
    // current_image: Option<&'a Array2<f64>>, // images: Vec<Array2<f64>>,
    camera: Array2<f64>,
}

impl<'a> DepthFilter<'a> {
    pub fn new(
        images: &'a Vec<Array2<f64>>,
        height: usize,
        width: usize,
        depth_mean: Option<f64>,
        depth_cov: Option<f64>,
        min_depth: Option<f64>,
        max_depth: Option<f64>,
        reader: Box<dyn Fn(&'a Vec<Array2<f64>>) -> ReaderResult<'a>>,
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
            images,
            height,
            width,
            depth_matrix: Array::from_elem((height, width), depth_mean),
            depth_cov2_matrix: Array::from_elem((height, width), depth_cov),
            min_depth,
            max_depth,
            reader, // images,
            // ref_image: None,
            // current_image: None,
            camera: array![[1., 0., 1.], [0., 1., 1.], [0., 0., 1.]],
        }
    }

    fn reader(&self) -> ReaderResult<'a> {
        let reader = &self.reader;
        reader(&self.images)
    }

    pub fn excute(&mut self) {
        let (ref_image, option_image, option_pose) = self.reader();

        if ref_image != None && option_image != None && option_pose != None {
            self.update(
                ref_image.unwrap(),
                option_image.unwrap(),
                &option_pose.unwrap(),
            );
            self.excute();
        }
    }

    /// 对整个深度图更新
    pub fn update(
        &mut self,
        ref_image: &Array2<f64>,
        current_image: &Array2<f64>,
        pose: &Array2<f64>,
    ) {
        for y in border..(self.height - border) {
            for x in border..(self.width - border) {
                if self.depth_cov2_matrix[[y, x]] < self.min_depth
                    || self.depth_cov2_matrix[[y, x]] > self.max_depth
                {
                    continue;
                }
                
                let mut pose = pose.clone().to_owned();
                pose.push(Axis(0), array![0., 0., 0., 1.].view())
                    .expect(&format!("pose {:?}", pose));
                let pt_ref = array![x as f64, y as f64];
                let depth = self.depth_matrix[[y, x]];
                let depth_cov = self.depth_cov2_matrix[[y, x]];
                if let Some((pt_curr, epiploar_direction)) =
                    self.epipolar_search(&pt_ref, ref_image, current_image, &pose, depth, depth_cov)
                {
                    // TODO: 更新深度图
                    self.update_depth_filter(
                        &vector![x as f64, y as f64],
                        &pt_curr,
                        &pose,
                        &epiploar_direction,
                    );
                }
            }
        }
    }

    /// 极线搜索
    /// @return (匹配点坐标， 极线方向)
    fn epipolar_search(
        &self,
        pt_ref: &Array1<f64>,
        ref_image: &Array2<f64>,
        current_image: &Array2<f64>,
        pose: &Array2<f64>,
        depth: f64,
        depth_cov: f64,
    ) -> Option<(Vector2<f64>, Vector2<f64>)> {
        let pose = pose.clone().into_nalgebra();

        let pt_world = px2cam(&vector![pt_ref[1], pt_ref[0]], &self.camera);
        let pt_world = vector![pt_world.x, pt_world.y, pt_world.z];
        let pt_world = pt_world.normalize() * depth;
        let pt_world = pose.slice((0, 0), (3, 3)) * (&pt_world) + pose.slice((0, 3), (3, 1));
        let pt_world = vector!(pt_world[(0, 0)], pt_world[(1, 0)], pt_world[(2, 0)]);
        let px_mean_curr = cam2px(&pt_world, &self.camera);
        let mut d_min = depth - 3. * depth_cov;
        let d_max = depth + 3. * depth_cov;
        if d_min < 0.1 {
            d_min = 0.1
        }
        let px_min_curr = cam2px(&(pt_world * d_min), &self.camera);
        let px_max_curr = cam2px(&(pt_world * d_max), &self.camera);
        let epipolar_line = px_max_curr - px_min_curr;
        let epipolar_direction = epipolar_line.normalize();
        let mut half_length = 0.5 * epipolar_line.norm();
        if half_length > 100. {
            half_length = 100.;
        }
        let mut best_ncc = -1.;
        let mut best_px_curr = vector![0., 0.];
        let mut l = -half_length;
        while l <= half_length {
            let px_curr = px_mean_curr + l * epipolar_direction;
            if !inside(&px_curr, self.width, self.height) {
                l = l + 0.7;
                continue;
            }
            let ncc_value = ncc(
                ref_image,
                current_image,
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
            Some((best_px_curr, epipolar_direction))
        }
    }

    /// 深度图更新
    /// 通过三角化， 进行深度图更新
    fn update_depth_filter(
        &mut self,
        pt_ref: &Vector2<f64>,
        pt_curr: &Vector2<f64>,
        pose: &Array2<f64>,
        epipolar_direction: &Vector2<f64>,
    ) {
        // let pose = pose.view().into_nalgebra();
        let f_ref = px2cam(pt_ref, &self.camera);
        let f_ref = f_ref.normalize();
        let f_curr = px2cam(pt_curr, &self.camera);
        let f_curr = f_curr.normalize();

        let R = pose.slice(s![0..3, 0..3]).to_owned();
        let R: DMatrix<f64> = R.into_nalgebra().try_inverse().unwrap();
        let t = vector![pose[[3, 0]], pose[[3, 1]], pose[[3, 2]]];

        let f2 = R * f_curr;
        let b = vector![(f_ref.transpose() * t).sum(), (&f2.transpose() * t).sum()];
        let A: Matrix2<f64> = Matrix2::new(
            (f_ref.transpose() * f_ref).sum(),
            (-f_ref.transpose() * &f2).sum(),
            (&f2.transpose() * f_ref).sum(),
            (-f2.transpose() * &f2).sum(),
        );
        let ans = A.try_inverse().unwrap() * b;

        let xm = ans[0] * f_ref;
        let xn = ans[1] * f2 + t;
        let p_esti = (xm + xn) / 2.;
        let dept_estimation = p_esti.norm();

        // 计算不确定性
        let p = f_ref * dept_estimation;
        let a = p - (t);
        let t_norm = t.norm();
        let a_norm = a.norm();
        let alpha = (f_ref.dot(&t) / t_norm).acos();
        // let beta = (-a.dot(&t) / (a_norm * t_norm)).acos();
        let f_curr_prim = px2cam(&(pt_curr + epipolar_direction), &self.camera);
        let f_curr_prim = f_curr_prim.normalize();
        let beta_prim = f_curr_prim.dot(&-t) / t_norm;
        let gamma = PI - alpha - beta_prim;
        let p_prim = t_norm * beta_prim.sin() / gamma.sin();
        let d_cov = p_prim - dept_estimation;
        let d_cov2 = d_cov * d_cov;

        //高斯融合
        let mu = self
            .depth_matrix
            .get((pt_ref.y as usize, pt_ref.x as usize))
            .unwrap();
        let sigma2 = self
            .depth_cov2_matrix
            .get((pt_ref.y as usize, pt_ref.x as usize))
            .unwrap();

        let mu_fuse = d_cov2 * mu + sigma2 * dept_estimation;
        let sigma_fuse2 = (sigma2 * d_cov2) / (sigma2 + d_cov2);

        self.depth_matrix[(pt_ref.y as usize, pt_ref.x as usize)] = mu_fuse;
        self.depth_cov2_matrix[(pt_ref.y as usize, pt_ref.x as usize)] = sigma_fuse2;
    }
}

/// 0：ref图像数据 1.curr图像数据 ， 2.pose矩阵
pub type ReaderResult<'b> = (
    Option<&'b Array2<f64>>,
    Option<&'b Array2<f64>>,
    Option<Array2<f64>>,
);

fn inside(pt: &Vector2<f64>, width: usize, height: usize) -> bool {
    let boarder: f64 = 20.;
    return pt[(0, 0)] >= boarder
        && pt[(1, 0)] >= boarder
        && pt[(0, 0)] + boarder < width as f64
        && pt[(1, 0)] + boarder <= height as f64;
}
