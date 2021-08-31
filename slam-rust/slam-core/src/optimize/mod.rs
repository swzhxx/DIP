use num_traits::Num;

/// Levenberg Marquardt Algorithm
pub struct LM {
    pub damp: f64,
    error: fn(args: &Vec<f64>) -> f64,
    jacobian: fn(args: &Vec<f64>) -> Vec<f64>,
}

impl LM {
    pub fn slove() -> Vec<f64> {
        todo!()
    }
}
