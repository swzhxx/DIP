use nalgebra::{DMatrix, Vector2};

pub trait BlockMatch {
    fn match_block(
        &self,
        image1: &DMatrix<f32>,
        image2: &DMatrix<f32>,
        pt1: &Vector2<u32>,
        pt2: &Vector2<u32>,
    ) -> f32;
}

#[derive(Debug, Clone)]
pub struct Ncc {
    ncc_window_size: Option<u32>,
}

impl Default for Ncc {
    fn default() -> Self {
        Self {
            ncc_window_size: Some(3),
        }
    }
}

impl BlockMatch for Ncc {
    fn match_block(
        &self,
        image1: &DMatrix<f32>,
        image2: &DMatrix<f32>,
        pt1: &Vector2<u32>,
        pt2: &Vector2<u32>,
    ) -> f32 {
        let ncc_window_size = self.ncc_window_size.unwrap_or(3);
        let block1 = image1.slice(
            (
                (pt1.x - ncc_window_size) as usize,
                (pt1.y - ncc_window_size) as usize,
            ),
            (
                (2 * ncc_window_size + 1) as usize,
                (2 * ncc_window_size + 1) as usize,
            ),
        );
        let block2 = image2.slice(
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
}
