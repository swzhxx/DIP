use nalgebra::{DMatrix, Vector2};

pub trait BlockMatch {
    fn match_block(&self, pt1: &Vector2<u32>, pt2: &Vector2<u32>) -> f32;
    fn better(&self, last_score: f32, current_score: f32) -> bool;
    fn best(&self, score: f32) -> bool;
}

#[derive(Debug, Clone)]
pub struct Ncc<'a> {
    ncc_window_size: Option<u32>,
    image1: &'a DMatrix<f32>,
    image2: &'a DMatrix<f32>,
    pass_number: f32,
}

impl<'a> Ncc<'a> {
    pub fn new(
        image1: &'a DMatrix<f32>,
        image2: &'a DMatrix<f32>,
        pass_number: f32,
        ncc_window_size: Option<u32>,
    ) -> Self {
        Self {
            image1,
            image2,
            pass_number,
            ncc_window_size,
        }
    }
}

impl BlockMatch for Ncc<'_> {
    fn match_block(&self, pt1: &Vector2<u32>, pt2: &Vector2<u32>) -> f32 {
        let ncc_window_size = self.ncc_window_size.unwrap_or(3);
        let block1 = self.image1.slice(
            (
                (pt1.x - ncc_window_size) as usize,
                (pt1.y - ncc_window_size) as usize,
            ),
            (
                (2 * ncc_window_size + 1) as usize,
                (2 * ncc_window_size + 1) as usize,
            ),
        );
        let block2 = self.image2.slice(
            (
                (pt2.x - ncc_window_size) as usize,
                (pt2.y - ncc_window_size) as usize,
            ),
            (
                (2 * ncc_window_size + 1) as usize,
                (2 * ncc_window_size + 1) as usize,
            ),
        );
        let block1_mean = block1.mean();
        let block2_mean = block2.mean();
        let block1 = block1.map(|item| item - block1_mean);
        let block2 = block2.map(|item| item - block2_mean);

        let numerator = (&block1 * &block2).sum();
        let divisor = ((block1.transpose() * &block1) * &(block2.transpose() * &block2))
            .sum()
            .sqrt();
        numerator / (divisor + 1e-18)
    }
    fn better(&self, last: f32, current: f32) -> bool {
        if current < last {
            true
        } else {
            false
        }
    }
    fn best(&self, score: f32) -> bool {
        if score < self.pass_number {
            true
        } else {
            false
        }
    }
}
