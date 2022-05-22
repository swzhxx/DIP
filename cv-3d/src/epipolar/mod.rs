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
    k1: &'a Matrix3<f64>,
    current_r_t: Matrix3x4<f64>,
    cov_value: f64,
    block_matcher: BlockMatcher,
}

impl<'a, BlockMatcher> EpipolarSearch<'a, BlockMatcher>
where
    BlockMatcher: BlockMatch,
{
    pub fn new(
        k1: &'a Matrix3<f64>,
        current_r_t: Matrix3x4<f64>,
        cov_value: f64,
        block_matcher: BlockMatcher,
    ) -> Self {
        if cov_value <= 1. {
            panic!("error epiploar search cov value {}", cov_value);
        }
        Self {
            current_r_t,
            k1,
            cov_value,
            block_matcher,
        }
    }
    pub fn search(
        &self,
        pt_current: &Vector2<u32>,
        depth: f64,
        depth_cov: f64,
        insider: &dyn Inside,
    ) -> (Option<Vector2<u32>>, Vector2<f64>) {
        if !insider.inside(&Vector2::new(pt_current.x as f64, pt_current.y as f64)) {
            return (None, Vector2::new(0., 0.));
        }
        let f_curr = px2cam(pt_current, self.k1);
        let f_curr = f_curr.normalize();
        let P_curr = f_curr * depth;

        let current_r = self.current_r_t.slice((0, 0), (3, 3));
        let current_t = self.current_r_t.slice((0, 3), (3, 1));

        let px_mean_curr = cam2px(
            &{
                let p = &current_r * &P_curr + &current_t;
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
                let p = &current_r * (&f_curr * d_min) + &current_t;
                Vector3::new(p[0], p[1], p[2])
            },
            self.k1,
        );
        let px_max_curr = cam2px(
            &{
                let p = &current_r * (&f_curr * d_max) + &current_t;
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
            if !insider.inside(&pt_next) {
                l += 0.7;
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
    fn inside(&self, pt: &Vector2<f64>) -> bool;
}

pub struct Insider<'a> {
    border: u32,
    image: &'a DMatrix<f64>,
}

impl<'a> Insider<'a> {
    pub fn new(border: u32, image: &'a DMatrix<f64>) -> Self {
        Self { border, image }
    }
}

impl<'a> Inside for Insider<'a> {
    fn inside(&self, pt: &Vector2<f64>) -> bool {
        let shape = self.image.shape();

        let border = self.border as f64;
        pt.x >= border
            && pt.y >= border
            && (pt.x + border) < shape.1 as f64
            && (pt.y + border) < shape.0 as f64
    }
}

#[cfg(test)]
mod test {
    use super::Insider;
    use crate::epipolar::Inside;
    use nalgebra::{DMatrix, Vector2};

    #[test]
    fn test_inside() {
        let image = DMatrix::from_element(640, 480, 1.);
        let insider = Insider::new(20, &image);
        assert_eq!(insider.inside(&Vector2::new(325., 477.)), false);
    }
}
