use nalgebra::DMatrix;

/// Frame
/// 目前只定义一个data
/// 更具后面的需求补充相应的参数
#[derive(Debug, Clone)]
pub struct Frame<'a> {
    pub data: &'a DMatrix<f64>,
}

// impl<'a, 'b> From<&'b DMatrix<f64>> for Frame<'a>
// where
//     'b: 'a,
// {
//     fn from(data: &'b DMatrix<f64>) -> Self {
//         Self { data }
//     }
// }

impl<'a, 'b> Into<Frame<'b>> for &'a Frame<'a>
where
    'a: 'b,
{
    fn into(self) -> Frame<'b> {
        Frame { data: &self.data }
    }
}

impl<'a, 'b> Into<Frame<'b>> for &'a DMatrix<f64>
where
    'a: 'b,
{
    fn into(self) -> Frame<'b> {
        Frame { data: self }
    }
}

impl<'a> Frame<'a> {
    pub fn new<'b>(data: &'b DMatrix<f64>) -> Self
    where
        'b: 'a,
    {
        Self { data }
    }

    pub fn clone(&self) -> Self {
        Self { data: self.data }
    }
}

impl Frame<'_> {
    pub fn shape(&self) -> (usize, usize) {
        self.data.shape()
    }

    pub fn get_data(&self) -> &DMatrix<f64> {
        self.data
    }
}
