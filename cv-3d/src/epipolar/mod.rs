use nalgebra::{DMatrix, Matrix3, Matrix3x4, Vector2, Vector3};

use crate::{
    block_match::{BlockMatch, Ncc},
    frame::Frame,
    utils::{cam2px, px2cam},
};

#[derive(Debug)]
pub struct EpipolarSearch<'a, BlockMatcher>
where
    BlockMatcher: BlockMatch,
{
    current_frame: Frame<'a>,
    next_frame: Frame<'a>,
    k1: &'a Matrix3<f32>,
    k2: &'a Matrix3<f32>,
    current_r_t: Matrix3x4<f32>,
    ref_r_t: Matrix3x4<f32>,
    cov_value: f32,
    block_matcher: BlockMatcher,
}

impl<'a, BlockMatcher> EpipolarSearch<'a, BlockMatcher>
where
    BlockMatcher: BlockMatch,
{
    pub fn new(
        current_frame: Frame<'a>,
        next_frame: Frame<'a>,
        k1: &'a Matrix3<f32>,
        k2: &'a Matrix3<f32>,
        current_r_t: Matrix3x4<f32>,
        ref_r_t: Matrix3x4<f32>,
        cov_value: f32,
        block_matcher: BlockMatcher,
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
            block_matcher,
        }
    }
    pub fn search(
        &self,
        pt_current: &Vector2<u32>,
        depth: f32,
        depth_cov: f32,
        insider: &dyn Inside,
    ) -> (Option<Vector2<u32>>, Vector2<f32>) {
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

        let epipolar_line = px_max_curr - px_min_curr;
        let epipolar_direction = epipolar_line.normalize();
        let mut half_length = 0.5 * epipolar_line.norm();
        // 不希望在极线搜索时，搜索太多的东西
        if half_length > 100. {
            half_length = 100.
        }

        let mut best_ncc = -1.;
        let mut best_px_curr: Option<Vector2<u32>> = None;

        let mut l = -half_length;
        while l < half_length {
            let pt_next = px_mean_curr + l * epipolar_direction;
            if insider.inside(&pt_next) {
                continue;
            }
            let pt_next = Vector2::new(pt_next.x as u32, pt_next.y as u32);
            let match_score = self.block_matcher.match_block(pt_current, &pt_next);
            if self.block_matcher.better(best_ncc, match_score) {
                best_ncc = match_score;
                best_px_curr = Some(pt_next);
            }

            l += 0.7;
        }
        (best_px_curr, epipolar_direction)
    }
}

pub trait Inside {
    fn inside(&self, pt: &Vector2<f32>) -> bool;
}

pub struct Insider<'a> {
    border: u32,
    image: &'a DMatrix<f32>,
}

impl<'a> Insider<'a> {
    pub fn new(border: u32, image: &'a DMatrix<f32>) -> Self {
        Self { border, image }
    }
}

impl<'a> Inside for Insider<'a> {
    fn inside(&self, pt: &Vector2<f32>) -> bool {
        let shape = self.image.shape();
        let border = self.border as f32;
        pt[(0, 0)] >= border
            && pt[(1, 0)] >= border
            && pt[(0, 0)] + border < shape.0 as f32
            && pt[(1, 0)] + border < shape.1 as f32
    }
}
