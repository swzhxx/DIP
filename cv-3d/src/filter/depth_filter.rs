use std::f64::consts::PI;

use nalgebra::{AbstractRotation, DMatrix, Matrix2, Matrix3, Matrix3x4, Vector2, Vector3};

use crate::{
    block_match::{BlockMatch, Ncc},
    epipolar::{self, EpipolarSearch, Insider},
    frame::Frame,
    utils::px2cam,
};

const min_cov: f64 = 0.1; // 收敛判定：最小方差
const max_cov: f64 = 10.; // 发散判定：最大方差

pub struct DepthFilter<'a> {
    initial: bool,
    current_frame: Frame<'a>,
    k1: &'a Matrix3<f64>,
    k2: &'a Matrix3<f64>,
    rotate: &'a Matrix3<f64>,
    translate: &'a Vector3<f64>,
    next_frame: Option<Frame<'a>>,
    depth_data: DMatrix<f64>,
    cov_data: DMatrix<f64>,
    epipolar_searcher: Option<EpipolarSearch<'a, Ncc<'a>>>,
}

impl<'a> DepthFilter<'a> {
    pub fn new<'b, T>(
        current_frame: T,
        init_depth: Option<f64>,
        k1: &'a Matrix3<f64>,
        k2: &'a Matrix3<f64>,
        rotate: &'a Matrix3<f64>,
        translate: &'a Vector3<f64>,
        cov_depth: Option<f64>,
    ) -> Self
    where
        'b: 'a,
        T: Into<Frame<'b>>,
    {
        let init_depth = init_depth.unwrap_or(3.);
        let cov_depth = cov_depth.unwrap_or(3.);

        let current_frame: Frame = current_frame.into();
        let shape = current_frame.shape();
        Self {
            initial: false,
            current_frame: current_frame,
            next_frame: None,
            depth_data: DMatrix::from_element(shape.0, shape.1, init_depth),
            cov_data: DMatrix::from_element(shape.0, shape.1, cov_depth),
            epipolar_searcher: None,
            k1,
            k2,
            rotate,
            translate,
        }
    }

    pub fn add_frame<T>(&mut self, next_frame: T)
    where
        T: Into<Frame<'a>>,
    {
        println!("add_frame");
        let next_frame = next_frame.into();
        if self.next_frame.is_none() {
            self.next_frame = Some(next_frame);
            self.init();
            self.update();
        } else {
            let frame = self.next_frame.take();
            self.current_frame = frame.unwrap();
            self.next_frame = Some(next_frame);
            self.update();
        }
    }

    fn init(&mut self) {
        self.initial = true;
        let ncc = Ncc::new(
            self.current_frame.clone(),
            self.next_frame.as_ref().unwrap().clone(),
            0.85,
            None,
        );

        let (_current_r_t, ref_r_t) = self.compute_current_ref_r_t();

        let epipolar_searcher = EpipolarSearch::new(self.k1, ref_r_t, 3., ncc);
        self.epipolar_searcher = Some(epipolar_searcher)
    }
}

impl DepthFilter<'_> {
    // 更新
    fn update(&mut self) {
        println!("depth filter update");
        let board = 20usize;
        let insider = Insider::new(board as u32, self.current_frame.data);
        let shape = self.current_frame.shape();
        let mut count = 0;
        for x in board..shape.1 {
            for y in 0..shape.0 {
                if self.cov_data[(y, x)] < min_cov || self.cov_data[(y, x)] > max_cov {
                    continue;
                }

                let pt_current = Vector2::new(x as u32, y as u32);
                let searcher = self.epipolar_searcher.as_ref().unwrap();
                let (best_pt, epipolar_direction) = searcher.search(
                    &pt_current,
                    self.depth_data[(y, x)],
                    self.cov_data[(y, x)],
                    &insider,
                );
                if best_pt.is_none() {
                    continue;
                }
                count += 1;
                if count % 10000 == 0 && count > 0 {
                    println!("ncc count {:?}", count);
                }
                self.update_depth_filter(
                    &pt_current,
                    best_pt.as_ref().unwrap(),
                    epipolar_direction,
                );
            }
        }
    }

    /// 计算depth
    ///
    /// 更新深度矩阵和深度的方差矩阵
    ///
    /// [三角化公式](https://www.cnblogs.com/Jessica-jie/p/7730731.html)
    fn update_depth_filter(
        &mut self,
        pt_current: &Vector2<u32>,
        pt_ref: &Vector2<u32>,
        epipolar_direction: Vector2<f64>,
    ) {
        let f_curr = px2cam(pt_current, self.k1);
        let f_curr = f_curr.normalize();
        let f_ref = px2cam(pt_ref, self.k2);
        let f_ref = f_ref.normalize();

        let f2 = self.rotate * &f_curr;
        let b = Vector2::new(self.translate.dot(&f_ref), self.translate.dot(&f2));

        let A = Matrix2::new(
            f_ref.dot(&f_ref),
            (-1. * &f_ref).dot(&f2),
            f_ref.dot(&f2),
            -f2.dot(&f2),
        );
        let ans = A.try_inverse().unwrap() * &b;
        let xm = ans[0] * f_ref;
        let xn = ans[1] * f2 + self.translate;
        let p_esti = (xm + xn) / 2.0;
        let depth_estimation = p_esti.norm();
        self.depth_data[(pt_current.y as usize, pt_current.x as usize)] = depth_estimation;
        // let inverse_depth_estimation = 1. / depth_estimation;

        //? 计算不确定性 , 这段完全看不懂
        let p = f_curr * depth_estimation;
        let a = p - self.translate;
        let t_norm = self.translate.norm();
        let a_norm = a.norm();
        let alpha = (f_ref.dot(&self.translate) / t_norm).acos();
        let _beta = (-a.dot(&self.translate) / (a_norm * t_norm)).acos();
        let f_ref_prime = px2cam(
            &(Vector2::new(pt_ref.x as f64, pt_ref.y as f64) + &epipolar_direction),
            self.k1,
        );
        let f_ref_prime = f_ref_prime.normalize();
        let beta_prime = (f_ref_prime.dot(&(-1. * self.translate)) / &t_norm).acos();
        let gamma = PI - alpha - beta_prime;
        let p_prime = t_norm * beta_prime.sin() / gamma.sin();
        let d_cov = p_prime - depth_estimation;
        let d_cov2 = d_cov * d_cov;

        // 高斯融合
        let mu = self.depth_data[((pt_current.y as usize), (pt_current.x as usize))];
        let sigma2 = self.cov_data[((pt_current.y as usize), (pt_current.x as usize))];
        let mu_fuse = (d_cov2 * mu + sigma2 * depth_estimation) / (sigma2 + d_cov2);
        let sigma_fuse2 = (sigma2 * &d_cov2) / (sigma2 + d_cov2);

        self.depth_data[(pt_current.y as usize, pt_current.x as usize)] = mu_fuse;
        self.cov_data[(pt_current.y as usize, pt_current.x as usize)] = sigma_fuse2;
    }

    fn compute_current_ref_r_t(&self) -> (Matrix3x4<f64>, Matrix3x4<f64>) {
        let current_r_t = Matrix3x4::identity();
        let mut r_t = self.rotate.clone().insert_column(3, 0.);
        r_t.column_mut(3).copy_from(self.translate);
        let mut ref_r_t = Matrix3x4::from_element(0.);
        ref_r_t.clone_from(&r_t);
        (current_r_t, r_t)
    }
}

impl DepthFilter<'_> {
    pub fn get_depth(&self) -> &DMatrix<f64> {
        return &self.depth_data;
    }
    pub fn get_depth_cov(&self) -> &DMatrix<f64> {
        return &self.cov_data;
    }
}
