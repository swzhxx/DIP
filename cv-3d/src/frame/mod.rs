use nalgebra::DMatrix;

/// Frame
/// 目前只定义一个data
/// 更具后面的需求补充相应的参数
#[derive(Debug)]
pub struct Frame<'a> {
    data: &'a DMatrix<f32>,
}

// impl<'a, 'b> From<&'b DMatrix<f32>> for Frame<'a>
// where
//     'b: 'a,
// {
//     fn from(data: &'b DMatrix<f32>) -> Self {
//         Self { data }
//     }
// }

impl<'a, 'b> Into<Frame<'b>> for &'a DMatrix<f32>
where
    'a: 'b,
{
    fn into(self) -> Frame<'b> {
        Frame { data: self }
    }
}

impl<'a> Frame<'a> {
    pub fn new<'b>(data: &'b DMatrix<f32>) -> Self
    where
        'b: 'a,
    {
        Self { data }
    }
}

impl Frame<'_> {
    pub fn shape(&self) -> (usize, usize) {
        self.data.shape()
    }

    pub fn get_data(&self) -> &DMatrix<f32> {
        self.data
    }
}
