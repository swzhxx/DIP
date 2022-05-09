use nalgebra::Vector2;
use opencv::core::KeyPoint;

pub mod block_match;
pub mod epipolar;
pub mod filter;
pub mod frame;
pub mod triangluate;
pub mod utils;
pub trait ToNaVector2 {
    fn to_vector2(&self) -> Vector2<f32>;
}

impl ToNaVector2 for KeyPoint {
    fn to_vector2(&self) -> Vector2<f32> {
        Vector2::new(self.pt.x, self.pt.y)
    }
}
