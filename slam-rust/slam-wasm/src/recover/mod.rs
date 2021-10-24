use std::{borrow::Borrow, cell::RefCell, rc::Rc};

use ndarray::{array, Array2, Axis};
use slam_core::{
    features::fast::OFast,
    filter::depth_filter::{DepthFilter, ReaderResult},
    matches::orb::Orb,
    point::{Point2, Point3},
    sfm::{find_pose, restoration_perspective_structure, EightPoint},
    triangulate::{self, Triangulate},
};
use wasm_bindgen::prelude::*;
use web_sys::ImageData;

use crate::utils::image_data_to_gray;

// #[wasm_bindgen]
// struct Recover3D {
//     pairs: Vec<Pair>,
//     transform_matrix: Option<Array2<f64>>,
// }

// #[wasm_bindgen]
// impl Recover3D {
//     ///  matches 是一个长度大于16并且为偶数的Vec
//     ///  x1,y1,x2,y2,x3,y3,x4,y4....
//     ///  其中1,2为配对点，3,4为配对点。
//     #[wasm_bindgen(constructor)]
//     pub fn new(matches: Vec<usize>) -> Self {
//         if matches.len() % 2 != 0 && matches.len() > 16 {
//             panic!("matches length must be odd and len must > 16")
//         }
//         let mut pairs: Vec<Pair> = vec![];

//         for (i, _) in matches.iter().enumerate() {
//             if i % 2 == 0 {
//                 continue;
//             }
//             if (i + 1) % 4 == 0 {
//                 continue;
//             }
//             let x1 = matches[i - 3];
//             let y1 = matches[i - 2];
//             let x2 = matches[i - 1];
//             let y2 = matches[i];
//             let point1 = Point2::new(x1, y1);
//             let point2 = Point2::new(x2, y2);
//             pairs.push((point1, point2));
//         }
//         Recover3D {
//             pairs,
//             transform_matrix: None,
//         }
//     }

//     pub fn compute_transform_matrix(&mut self) {
//         let matches1: Vec<Point2<f64>> = self
//             .pairs
//             .iter()
//             .map(|pair| Point2::new(pair.0.x as f64, pair.0.y as f64))
//             .collect();

//         let matches2: Vec<Point2<f64>> = self
//             .pairs
//             .iter()
//             .map(|pair| Point2::new(pair.1.x as f64, pair.1.y as f64))
//             .collect();
//         let mut eight_points = EightPoint::new(&matches1, &matches2);
//         let fundamental = eight_points.normalize_find_fundamental();
//         if fundamental == None {
//             return;
//         }
//         let fundamental = fundamental.unwrap();
//         let mut transform_matrix =
//             restoration_perspective_structure(&fundamental, &matches1, &matches2, None);
//         transform_matrix.push(Axis(0), array![0., 0., 0., 1.].view());
//         self.transform_matrix = Some(transform_matrix);
//     }

//     /// 恢复3维点坐标
//     /// shape[0]height， shape[1]width
//     pub fn recover_3d_coordinate(shape: Vec<usize>) {
//         if shape.len() != 2 {
//             panic!("shape length must be 2")
//         }
//     }
// }

#[wasm_bindgen]
struct Recover3D {
    images: Vec<Array2<f64>>,
    map_3d: Option<Vec<f64>>,
}

#[wasm_bindgen]
impl Recover3D {
    #[wasm_bindgen(constructor)]
    pub fn new(images: Vec<ImageData>) -> Self {
        let images = images
            .iter()
            .map(|image: &ImageData| {
                let width = image.width() as usize;
                let height = image.height() as usize;
                let gray_image = Array2::from_shape_vec(
                    (height, width),
                    image_data_to_gray(image)
                        .into_iter()
                        .map(|v| v as f64)
                        .collect(),
                )
                .unwrap();
                gray_image
            })
            .collect();
        Recover3D {
            images,
            map_3d: None,
        }
    }

    pub fn recover(&mut self) {
        let mut images = vec![];
        std::mem::swap(&mut self.images, &mut images);

        let rc_images = Rc::new(images);
        let ref_image = &rc_images[0];
        let shape = ref_image.shape();
        let rc_images_clone = Rc::clone(&rc_images);
        let i = RefCell::new(1);
        // let i_borrow_mut = i.borrow_mut();

        let ref_features = OFast::new(ref_image).find_features(None);
        let ref_descriptors = Orb::new(ref_image, &ref_features).create_descriptors();

        let reader: Box<dyn for<'a> Fn(&'a dp) -> ReaderResult<'a>> = Box::new(move || {
            if rc_images_clone.len() == 0 || rc_images_clone.len() == 1 {
                return (None, None);
            }
            let curr_image = &rc_images_clone[*i.borrow()];
            let curr_features = OFast::new(curr_image).find_features(None);
            let curr_descriptors = Orb::new(curr_image, &curr_features).create_descriptors();
            let mut matches = Orb::brief_match(&ref_descriptors, &curr_descriptors, 40);
            let matches1 = matches
                .iter()
                .map(|dmatch| ref_features[dmatch.i1].clone().f())
                .collect();
            let matches2 = matches
                .iter()
                .map(|dmatch| curr_features[dmatch.i2].clone().f())
                .collect();
            let fundamental = EightPoint::new(&matches1, &matches2).normalize_find_fundamental();
            if fundamental == None {
                *i.borrow_mut() = *i.borrow() + 1;
                return (None, None);
            }
            let pose = restoration_perspective_structure(
                &fundamental.unwrap(),
                &matches1,
                &matches2,
                None,
            );
            *i.borrow_mut() = *i.borrow() + 1;
            return (Some(curr_image), Some(pose));
        });
        let mut depth_filter = DepthFilter::new(shape[0], shape[1], None, None, None, None, reader);
        depth_filter.excute();

        // 还原3维坐
        todo!()
    }
}
