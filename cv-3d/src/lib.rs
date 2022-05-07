use nalgebra::Vector2;
use opencv::core::KeyPoint;

pub mod triangluate;

pub trait ToNaVector2 {
    fn to_vector2(&self) -> Vector2<f32>;
}

impl ToNaVector2 for KeyPoint {
    fn to_vector2(&self) -> Vector2<f32> {
        Vector2::new(self.pt.x, self.pt.y)
    }
}
