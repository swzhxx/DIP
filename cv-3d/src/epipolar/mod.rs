use nalgebra::{Matrix3, Matrix3x4, Vector2, Vector3};

use crate::{
    frame::Frame,
    utils::{cam2px, px2cam},
};

#[derive(Debug)]
pub struct EpipolarSearch<'a> {
    current_frame: &'a Frame<'a>,
    next_frame: &'a Frame<'a>,
    k1: &'a Matrix3<f32>,
    k2: &'a Matrix3<f32>,
    current_r_t: &'a Matrix3x4<f32>,
    ref_r_t: &'a Matrix3x4<f32>,
    cov_value: f32,
}

impl<'a> EpipolarSearch<'a> {
    pub fn new(
        current_frame: &'a Frame,
        next_frame: &'a Frame,
        k1: &'a Matrix3<f32>,
        k2: &'a Matrix3<f32>,
        current_r_t: &'a Matrix3x4<f32>,
        ref_r_t: &'a Matrix3x4<f32>,
        cov_value: f32,
    ) -> Self {
        if cov_value <= 1. {
            panic!("error epiploar search cov value {}", cov_value);
        }
        Self {
            current_frame,
            next_frame,
            current_r_t,
            ref_r_t,
            k1,
            k2,
            cov_value,
        }
    }
    pub fn search(
        &self,
        pt_current: &Vector2<u32>,
        pt_next: &Vector2<u32>,
        depth: f32,
        max_depth: f32,
        min_depth: f32,
        depth_cov: f32,
    ) -> Option<Vector2<u32>> {
        let f_ref = px2cam(pt_current, self.k1);
        let f_ref = f_ref.normalize();
        let P_ref = f_ref * depth;
        let current_r = self.current_r_t.slice((0, 0), (3, 3));
        let current_t = self.current_r_t.slice((3, 0), (3, 1));

        let px_mean_curr = cam2px(
            &{
                let p = &current_r * &P_ref + &current_t;
                Vector3::new(p[0], p[1], p[2])
            },
            self.k1,
        );
        let mut d_min = depth - self.cov_value * depth_cov;
        let d_max = depth + self.cov_value * depth_cov;
        if d_min < 0.1 {
            d_min = 0.1
        }
        let px_min_curr = cam2px(
            &{
                let p = &current_r * (&f_ref * d_min) + &current_t;
                Vector3::new(p[0], p[1], p[2])
            },
            self.k1,
        );
        let px_max_curr = cam2px(
            &{
                let p = &current_r * (&f_ref * d_max) + &current_t;
                Vector3::new(p[0], p[1], p[2])
            },
            self.k1,
        );
        todo!()
    }
}
