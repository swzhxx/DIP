use ndarray::{s, Array1, Array2, ArrayD, ArrayView, ArrayView1, ArrayView2, Axis};
use ndarray_stats::QuantileExt;

use nshare::ToNalgebra;
use num_traits::{Float, Pow};

use std::rc::Rc;
pub type ErrorClousure<T> = Rc<Box<dyn Fn(&ArrayView1<f64>, usize, &T, &T) -> f64>>;
pub type JacobianClousure<T> = Rc<Box<dyn Fn(&ArrayView1<f64>, usize, &T) -> Array1<f64>>>;
/// Levenberg Marquardt Algorithm
pub struct LM<'a, T> {
    tao: Option<f64>,
    damp: Option<f64>,
    inputs: &'a Vec<T>,
    outputs: &'a Vec<T>,
    error_closure: ErrorClousure<T>,
    jacobian_closure: JacobianClousure<T>,
    v: usize,
}

impl<'a, T> LM<'a, T> {
    /// tao 设置为0时，LM算法近乎等价GaussianNewtoon算法
    pub fn new(
        inputs: &'a Vec<T>,
        outputs: &'a Vec<T>,
        error_closure: ErrorClousure<T>,
        jacobian_closure: JacobianClousure<T>,
        tao: Option<f64>,
    ) -> Self {
        let tao = match tao {
            Some(val) => Some(val),
            None => Some(1.),
        };
        LM {
            damp: None,
            inputs,
            outputs,
            error_closure,
            jacobian_closure,
            tao,
            v: 2 as usize,
        }
    }

    /// 初始化阻尼系数
    fn init_damp(&mut self, a: &Array2<f64>) {
        let h = a.dot(&a.t());
        let max = h.diag().max().unwrap().clone();
        self.damp = Some(max * self.tao.unwrap() + 1e-8);

        // self.damp = Some(1.);
    }

    /// 优化
    ///
    /// *args*:初始化参数，
    ///
    /// *max_iter_count*:最大的迭代次数
    ///
    /// *upsilon*:当差值低于该值时，终止迭代
    ///
    ///
    pub fn optimize(
        &mut self,
        args: &Array1<f64>,
        max_iter_count: Option<usize>,
        upsilon: Option<f64>,
    ) -> Array1<f64> {
        let shape = args.shape();
        let max_iter_count = match max_iter_count {
            Some(val) => val,
            None => 1000,
        };

        let upsilon = match upsilon {
            Some(val) => val,
            None => 1e-10,
        };

        let mut fitting = args.to_shape((shape[0], 1)).unwrap().to_owned();
        let len = fitting.len();
        // let mut last_cost: Option<f64> = None;
        let mut count = 0;
        loop {
            let reshape_fitting = fitting.slice(s![.., ..]).to_shape(len).unwrap().to_owned();
            let errors = self.error(&reshape_fitting.slice(s![..]));
            let jaco = self.jaco(&reshape_fitting.slice(s![..]));
            let cost = self.cost(&errors.view());
            match self.damp {
                None => {
                    self.init_damp(&fitting);
                }
                _ => {}
            }

            if let Some(temp) = self.update(&jaco, &errors, &fitting, upsilon) {
                fitting = temp;
            }

            if cost < upsilon {
                break;
            }
            count = count + 1;
            if count >= max_iter_count {
                break;
            }
        }
        let len = fitting.len();
        fitting.into_shape(len).unwrap().to_owned()
    }

    /// [列文伯格算法参数更新](https://zhuanlan.zhihu.com/p/42415718)
    fn update(
        &mut self,
        j: &Array2<f64>,
        f: &Array2<f64>,
        a: &Array2<f64>,
        upsilon: f64,
    ) -> Option<Array2<f64>> {
        let hessian: Array2<f64> = j.t().dot(j);
        let b = -(j.t().dot(f));
        let eye = Array2::eye(a.len());
        let delta_x = {
            let reshpe_b = b.to_shape(b.len()).unwrap();
            let h = (&hessian.view() + self.damp.unwrap() * eye).into_nalgebra();
            let rb = reshpe_b.view().into_nalgebra();
            let decomp = h.cholesky().unwrap();
            let mut result: Vec<f64> = vec![];
            for val in decomp.solve(&rb).into_iter() {
                result.push(*val);
            }
            Array2::<f64>::from_shape_vec((result.len(), 1), result).unwrap()
        };

        let cost = self.cost(&f.view());
        let next_arg = &delta_x + a;
        let re_next_arg = next_arg.to_shape(a.len()).unwrap().to_owned();
        let next_cost = self.cost(&self.error(&re_next_arg.slice(s![..])).view());
        if next_cost < upsilon {
            return Some(next_arg);
        }
        let rho = {
            let actual = cost - next_cost;
            let predictual = -(delta_x.t().dot(&j.t()).dot(f)
                + 0.5 * delta_x.t().dot(&hessian).dot(&delta_x))
            .sum();

            actual / (predictual + 1e-8)
        };

        if rho > 0. {
            // update dampe
            self.damp = Some(self.damp.unwrap() * (0.33).max(1. - (2. * rho - 1.).pow(3)));
            self.v = 2;
            let updated = a + &delta_x;

            Some(updated)
        } else {
            self.damp = Some(self.damp.unwrap() * (self.v as f64));
            self.v = self.v * 2;
            None
        }
    }

    /// error 矩阵
    fn error(&self, args: &ArrayView1<f64>) -> Array2<f64> {
        let mut errors = vec![];
        let error_closure = self.error_closure.clone();
        for row_i in 0..self.inputs.len() {
            let error = error_closure(args, row_i, &self.inputs[row_i], &self.outputs[row_i]);
            errors.push(error);
        }
        Array2::from_shape_vec((errors.len(), 1), errors).unwrap()
    }

    /// cost
    fn cost(&self, error_matrix: &ArrayView2<f64>) -> f64 {
        0.5 * error_matrix.dot(&error_matrix.t()).sum()
    }
    /// 求解jacobian
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
        let real = Array1::from_shape_vec(3, vec![1., 5., 2.]).unwrap();
        let fitting = Array1::from_shape_vec(3, vec![0., 0., 0.]).unwrap();
        let inputs = {
            let mut v = vec![];
            for k in 0..100 {
                v.push(k as f64 / 100.);
            }
            v
        };

        let outputs: Vec<f64> = inputs
            .iter()
            .map(|input| return (input * input * real[0] + input * real[1] + real[2]).exp())
            .collect();

        let mut lm: LM<f64> = LM::new(
            &inputs,
            &outputs,
            Rc::new(Box::new(
                |args: &ArrayView1<f64>, row_i: usize, input, output| -> f64 {
                    let input = input;
                    let error =
                        output - (input * input * args[0] + input * args[1] + args[2]).exp();

                    error
                },
            )),
            Rc::new(Box::new(
                |args: &ArrayView1<f64>, row_i: usize, input| -> Array1<f64> {
                    let a = -(input * input)
                        * (input * input * args[0] + input * args[1] + args[2]).exp();
                    let b = -input * (input * input * args[0] + input * args[1] + args[2]).exp();
                    let c = -(input * input * args[0] + input * args[1] + args[2]).exp();

                    Array1::from_vec(vec![a, b, c])
                },
            )),
            Some(0.),
        );

        let fitting = lm.optimize(&fitting, Some(10000), Some(1e-6));
        println!("optmize arguments {:?}", fitting)
    }
}
