use ndarray::{s, Array, Array1, Array2, ArrayD, ArrayView, ArrayView1, Axis};
use ndarray_stats::QuantileExt;

use nshare::{ToNalgebra, ToNdarray2};

use std::{ops::Mul, rc::Rc};
pub type ErrorClousure<T> = Rc<Box<dyn Fn(&ArrayView1<f64>, usize, &T, &T) -> f64>>;
pub type JacobianClousure<T> = Rc<Box<dyn Fn(&ArrayView1<f64>, usize, &T) -> Array1<f64>>>;
/// Levenberg Marquardt Algorithm
pub struct LM<'a, T> {
    damp: Option<f64>,
    inputs: &'a Vec<T>,
    outputs: &'a Vec<T>,
    error_closure: ErrorClousure<T>,
    jacobian_closure: JacobianClousure<T>,
}

impl<'a, T> LM<'a, T> {
    fn new(
        inputs: &'a Vec<T>,
        outputs: &'a Vec<T>,
        error_closure: ErrorClousure<T>,
        jacobian_closure: JacobianClousure<T>,
    ) -> Self {
        LM {
            damp: None,
            inputs,
            outputs,
            error_closure,
            jacobian_closure,
        }
    }

    /// 初始化阻尼系数
    fn init_damp(&mut self, j: &Array2<f64>) {
        let h = j.dot(&j.t());
        let max = h.diag().max().unwrap().clone();
        self.damp = Some(max * 0.001);
    }

    pub fn optimize(&mut self, args: &Array1<f64>, upsilon: f64) -> Array1<f64> {
        let shape = args.shape();
        let mut fitting = args.to_shape((shape[0], 1)).unwrap().to_owned();

        let mut last_cost: Option<f64> = None;
        loop {
            let errors = self.error(&args.slice(s![..]));
            let jaco = self.jaco(&args.slice(s![..]));
            let cost = errors.sum();
            match self.damp {
                None => {
                    self.init_damp(&jaco);
                }
                _ => {}
            }
            fitting = self.update(&jaco, &errors, &fitting);
            match last_cost {
                Some(lc) => {
                    if (lc - cost).abs() < upsilon.abs() {
                        break;
                    }
                    last_cost = Some(cost);
                }
                None => last_cost = Some(cost),
            }
        }
        let len = fitting.len();
        fitting.into_shape(len).unwrap().to_owned()
    }

    /// [列文伯格算法参数更新](https://zhuanlan.zhihu.com/p/42415718)
    fn update(&mut self, j: &Array2<f64>, f: &Array2<f64>, a: &Array2<f64>) -> Array2<f64> {
        let hessian: Array2<f64> = j.t().dot(j);
        let b = -j.t().dot(f);
        let delta_x = {
            let reshpe_b = b.to_shape(b.len()).unwrap();
            let h = hessian.view().into_nalgebra();
            let rb = reshpe_b.view().into_nalgebra();
            let decomp = h.cholesky().unwrap();
            let mut result: Vec<f64> = vec![];
            for val in decomp.solve(&rb).into_iter() {
                result.push(*val);
            }
            Array2::<f64>::from_shape_vec((result.len(), 1), result).unwrap()
            // hessian.solve(&reshpe_b).unwrap()
        };
        let cost = a.sum();

        let rho = {
            let next_arg = &delta_x + a;
            let re_next_arg = next_arg.to_shape((a.len())).unwrap().to_owned();
            let next_cost = self.error(&re_next_arg.slice(s![..])).sum();

            let actual = (cost - next_cost).abs();
            let predictual =
                (delta_x.t().dot(j).dot(f) + 0.5 * delta_x.t().dot(&hessian).dot(&delta_x)).sum();
            actual / (predictual + 1e-18)
        };
        if rho < 0.25 {
            self.damp = Some(self.damp.unwrap() * 2.0);
        } else if rho > 0.75 {
            self.damp = Some(self.damp.unwrap() / 3.0);
        }
        a - rho * &delta_x
    }

    /// error 计算
    fn error(&self, args: &ArrayView1<f64>) -> Array2<f64> {
        let mut errors = vec![];
        let error_closure = self.error_closure.clone();
        for row_i in 0..self.inputs.len() {
            let error = error_closure(args, row_i, &self.inputs[row_i], &self.outputs[row_i]);
            errors.push(error);
        }
        Array2::from_shape_vec((errors.len(), 1), errors).unwrap()
    }

    fn jaco(&self, args: &ArrayView1<f64>) -> Array2<f64> {
        let mut jaco = Array2::<f64>::zeros((0, args.len()));
        let jacobian_closure = self.jacobian_closure.clone();
        for row_i in 0..self.inputs.len() {
            let j = jacobian_closure(args, row_i, &self.inputs[row_i]);
            jaco.push(Axis(0), j.slice(s![..]));
        }
        jaco
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ndarray::{Array1, ArrayView0};
    #[test]
    fn test_lm_algoritm() {
        let real = Array1::from_shape_vec(3, vec![1.0, 2.0, 1.0]).unwrap();
        let mut fitting = Array1::from_shape_vec(3, vec![2.0, -1.0, 5.0]).unwrap();
        let inputs = {
            let mut v = vec![];
            for k in 0..100 {
                v.push(k as f64 / 100.);
            }
            v
        };

        let outputs: Vec<f64> = inputs
            .iter()
            .map(|input| return (input * real[0] * real[0] + input * real[1] + real[2]).exp())
            .collect();

        let mut lm: LM<f64> = LM::new(
            &inputs,
            &outputs,
            Rc::new(Box::new(
                |args: &ArrayView1<f64>, row_i: usize, input, output| -> f64 {
                    let input = input;
                    let error =
                        output - (input * args[0] * args[0] + input * args[1] + args[2]).exp();
                    error
                },
            )),
            Rc::new(Box::new(
                |args: &ArrayView1<f64>, row_i: usize, input| -> Array1<f64> {
                    let a = -input
                        * input
                        * (args[0] * args[0] * input + args[1] * input + args[2]).exp();
                    let b = -input * (args[0] * args[0] * input + args[1] * input + args[2]).exp();
                    let c = -(args[0] * args[0] * input + args[1] * input + args[2]).exp();
                    Array1::from_vec(vec![a, b, c])
                },
            )),
        );

        let fitting = lm.optimize(&fitting, 0.0001);
        eprintln!("{:?}", fitting)
    }
}
