use std::arch::x86_64::_MM_FROUND_RAISE_EXC;

use ndarray::{array, Array2, Axis};
use slam_core::{
    point::{Point2, Point3},
    sfm::{find_pose, restoration_perspective_structure, EightPoint},
    triangulate::{self, Triangulate},
};
use wasm_bindgen::prelude::*;

type Pair = (Point2<usize>, Point2<usize>);

#[wasm_bindgen]
struct Recover3D {
    pairs: Vec<Pair>,
    transform_matrix: Option<Array2<f64>>,
}

#[wasm_bindgen]
impl Recover3D {
    ///  matches 是一个长度大于16并且为偶数的Vec
    ///  x1,y1,x2,y2,x3,y3,x4,y4....
    ///  其中1,2为配对点，3,4为配对点。
    #[wasm_bindgen(constructor)]
    pub fn new(matches: Vec<usize>) -> Self {
        if matches.len() % 2 != 0 && matches.len() > 16 {
            panic!("matches length must be odd and len must > 16")
        }
        let mut pairs: Vec<Pair> = vec![];

        for (i, _) in matches.iter().enumerate() {
            if i % 2 == 0 {
                continue;
            }
            if (i + 1) % 4 == 0 {
                continue;
            }
            let x1 = matches[i - 3];
            let y1 = matches[i - 2];
            let x2 = matches[i - 1];
            let y2 = matches[i];
            let point1 = Point2::new(x1, y1);
            let point2 = Point2::new(x2, y2);
            pairs.push((point1, point2));
        }
        Recover3D {
            pairs,
            transform_matrix: None,
        }
    }

    pub fn compute_transform_matrix(&mut self) {
        let matches1: Vec<Point2<f64>> = self
            .pairs
            .iter()
            .map(|pair| Point2::new(pair.0.x as f64, pair.0.y as f64))
            .collect();

        let matches2: Vec<Point2<f64>> = self
            .pairs
            .iter()
            .map(|pair| Point2::new(pair.1.x as f64, pair.1.y as f64))
            .collect();
        let mut eight_points = EightPoint::new(&matches1, &matches2);
        let fundamental = eight_points.normalize_find_fundamental();
        if fundamental == None {
            return;
        }
        let fundamental = fundamental.unwrap();
        let mut transform_matrix =
            restoration_perspective_structure(&fundamental, &matches1, &matches2, None);
        transform_matrix.push(Axis(0), array![0., 0., 0., 1.].view());
        self.transform_matrix = Some(transform_matrix);
    }

    /// 恢复3维点坐标
    /// shape[0]height， shape[1]width
    pub fn recover_3d_coordinate(shape: Vec<usize>) {
        if shape.len() != 2 {
            panic!("shape length must be 2")
        }
    }
}
