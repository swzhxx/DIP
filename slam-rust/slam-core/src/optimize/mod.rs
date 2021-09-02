use std::{ops::Mul, rc::Rc};

use ndarray::{s, Array, Array1, Array2, ArrayD, ArrayView, ArrayView1, Axis};
use ndarray_linalg::Solve;
use ndarray_stats::QuantileExt;
use num_traits::Num;

/// Levenberg Marquardt Algorithm
pub struct LM {
    damp: Option<f64>,
    datas: usize,
    error_closure: Rc<Box<dyn Fn(&ArrayView1<f64>, usize) -> f64>>,
    jacobian_closure: Rc<Box<dyn Fn(&ArrayView1<f64>, usize) -> Array1<f64>>>,
}

impl LM {
    /// 初始化阻尼系数
    fn init_damp(&mut self, j: &Array2<f64>) {
        let h = j.dot(&j.t());
        let max = h.diag().max().unwrap().clone();
        self.damp = Some(max);
    }

    pub fn slove(&mut self, args: &Array1<f64>, stop_delta: f64) -> Array1<f64> {
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
                    if (lc - cost).abs() < stop_delta.abs() {
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
        let hessian = j.t().dot(j);
        let b = -j.t().dot(f);
        let delta_x = {
            let reshpe_b = b.to_shape(b.len()).unwrap();
            hessian.solve(&reshpe_b).unwrap()
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
        for row_i in 0..self.datas {
            let error = error_closure(args, row_i);
            errors.push(error);
        }
        Array2::from_shape_vec((errors.len(), 1), errors).unwrap()
    }

    fn jaco(&self, args: &ArrayView1<f64>) -> Array2<f64> {
        let mut jaco = Array2::<f64>::zeros((0, args.len()));
        let jacobian_closure = self.jacobian_closure.clone();
        for row_i in 0..self.datas {
            let j = jacobian_closure(args, row_i);
            jaco.push(Axis(0), j.slice(s![..]));
        }
        jaco
    }
}
