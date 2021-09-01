use std::{ops::Mul, rc::Rc};

use ndarray::{Array, Array1, Array2, ArrayD};
use ndarray_linalg::Solve;
use num_traits::Num;

/// Levenberg Marquardt Algorithm
pub struct LM {
    damp: f64,
    error: Rc<Box<dyn Fn(&Array2<f64>) -> Array2<f64>>>,
    jacobian: Rc<Box<dyn Fn(&Array2<f64>, &Array2<f64>) -> Array2<f64>>>,
}

impl LM {
    fn init_damp(&mut self) {}

    pub fn slove(&mut self, args: &Array2<f64>, stop_delta: f64) -> Array2<f64> {
        self.init_damp();
        let shape = args.shape();
        let mut O = args.clone();
        let error_fn = self.error.clone();
        let jacobian_fn = self.jacobian.clone();
        let mut last_cost: Option<f64> = None;
        loop {
            let E = error_fn(&O);
            let J = jacobian_fn(&E, &O);
            let cost = E.sum();
            let ref_O = O;
            O = self.update(&J, &E, ref_O);
            match last_cost {
                Some(lc) => {
                    if (lc - cost).abs() < stop_delta.abs() {
                        break;
                    }
                    last_cost = Some(cost);
                }
                None => {
                    last_cost = Some(cost);
                }
            }
        }
        O
    }

    /// [列文伯格算法参数更新](https://zhuanlan.zhihu.com/p/42415718)
    fn update(&mut self, J: &Array2<f64>, E: &Array2<f64>, A: Array2<f64>) -> Array2<f64> {
        let u = self.damp;
        let H = J.t().dot(J);
        let I = Array::from_shape_fn(H.dim(), |(i, j)| if i == j { 1.0 } else { 0. });

        let b = J.t().dot(&E.t());
        let b = b.to_shape((b.shape()[0])).unwrap();

        let X = (&H + u * I).solve(&b).unwrap();
        let Dx = &A - &X;
        let err = E.sum();
        let error_fn = self.error.clone();
        let new_err = error_fn(&(&X + &Dx)).sum();
        let rho = {
            let f = E.to_shape(E.shape()[0]).unwrap();
            let h = Dx.to_shape(Dx.shape()[0]).unwrap();
            (new_err - err).abs()
                / (0.5 * &f.t().dot(&f) + h.t().dot(J).dot(&f) + 0.5 * h.t().dot(&H).dot(&h))
        };
        if rho < 0.25 {
            self.damp = self.damp * 2.0;
        } else if rho > 0.75 {
            self.damp = self.damp / 3.0;
        }
        let O = &A - rho * &Dx;
        O
    }
}
