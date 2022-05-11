use nalgebra::{DMatrix, Matrix3, Matrix3x4, Vector2, Vector3};

use crate::{
    block_match::{BlockMatch, Ncc},
    epipolar::{self, EpipolarSearch, Insider},
    frame::Frame,
};

const min_cov: f32 = 0.1; // 收敛判定：最小方差
const max_cov: f32 = 10.; // 发散判定：最大方差

pub struct DepthFilter<'a> {
    initial: bool,
    current_frame: Frame<'a>,
    k1: &'a Matrix3<f32>,
    k2: &'a Matrix3<f32>,
    rotate: &'a Matrix3<f32>,
    translate: &'a Vector3<f32>,
    next_frame: Option<Frame<'a>>,
    depth_data: DMatrix<f32>,
    cov_data: DMatrix<f32>,
    epipolar_searcher: Option<EpipolarSearch<'a, Ncc<'a>>>,
}

impl<'a> DepthFilter<'a> {
    fn new<'b, T>(
        current_frame: T,
        init_depth: Option<f32>,
        k1: &'a Matrix3<f32>,
        k2: &'a Matrix3<f32>,
        rotate: &'a Matrix3<f32>,
        translate: &'a Vector3<f32>,
        cov_depth: Option<f32>,
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

    fn add_frame<T>(&mut self, next_frame: T)
    where
        T: Into<Frame<'a>>,
    {
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

        let (current_r_t, ref_r_t) = self.compute_current_ref_r_t();

        let epipolar_searcher = EpipolarSearch::new(
            self.current_frame.clone(),
            self.next_frame.as_ref().unwrap().clone(),
            self.k1,
            self.k2,
            current_r_t,
            ref_r_t,
            3.,
            ncc,
        );
        self.epipolar_searcher = Some(epipolar_searcher)
    }
}

impl DepthFilter<'_> {
    // 极线搜索
    fn epipolar_search(&self) {
        todo!()
    }
    // 更新
    fn update(&mut self) {
        let board = 20usize;
        let insider = Insider::new(board as u32, self.current_frame.data);
        let shape = self.current_frame.shape();

        let (_current_r_t, ref_r_t) = self.compute_current_ref_r_t();
        for x in board..shape.0 {
            for y in 0..shape.1 {
                if self.cov_data[(x, y)] < min_cov || self.cov_data[(x, y)] > max_cov {
                    continue;
                }
                let pt_current = Vector2::new(x as u32, y as u32);
                let searcher = self.epipolar_searcher.as_ref().unwrap();
                let (best_pt, epipolar_direction) = searcher.search(
                    &pt_current,
                    self.depth_data[(x, y)],
                    self.cov_data[(x, y)],
                    &insider,
                );
                if best_pt.is_none() {
                    continue;
                }
                self.update_depth_filter(
                    &pt_current,
                    best_pt.as_ref().unwrap(),
                    epipolar_direction,
                    ref_r_t,
                );
            }
        }
        todo!()
    }

    /// 计算depth
    /// 更新深度矩阵和深度的方差矩阵

    fn update_depth_filter(
        &mut self,
        pt_current: &Vector2<u32>,
        pt_ref: &Vector2<u32>,
        epipolar_direction: Vector2<f32>,
        r_t: Matrix3x4<f32>,
    ) {
        todo!()
    }

    fn compute_current_ref_r_t(&self) -> (Matrix3x4<f32>, Matrix3x4<f32>) {
        let current_r_t = Matrix3x4::identity();
        let mut r_t = self.rotate.clone().insert_column(3, 0.);
        r_t.column_mut(3).copy_from(self.translate);
        let mut ref_r_t = Matrix3x4::from_element(0.);
        ref_r_t.clone_from(&r_t);
        (current_r_t, r_t)
    }
}
