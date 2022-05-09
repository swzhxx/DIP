use nalgebra::DMatrix;

use crate::frame::Frame;
pub struct DepthFilter<'a> {
    initial: bool,
    current_frame: Frame<'a>,
    next_frame: Option<Frame<'a>>,
    depth_data: DMatrix<f32>,
    cov_data: DMatrix<f32>,
}

impl<'a> DepthFilter<'a> {
    fn new<'b, T>(current_frame: T, init_depth: Option<f32>, cov_depth: Option<f32>) -> Self
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
        }
    }

    fn add_frame<T>(&mut self, next_frame: T)
    where
        T: Into<Frame<'a>>,
    {
        let next_frame = next_frame.into();
        if self.next_frame.is_none() {
            self.next_frame = Some(next_frame);
            self.init()
        } else {
            let frame = self.next_frame.take();
            self.current_frame = frame.unwrap();
            self.next_frame = Some(next_frame)
        }
    }

    fn init(&mut self) {
        self.initial = true
    }
}

impl DepthFilter<'_> {
    // 极线搜索
    // fn epipolar_search(&self) {
    //     todo!()
    // }
    // 更新
    fn update(&mut self) {
        todo!()
    }
}
