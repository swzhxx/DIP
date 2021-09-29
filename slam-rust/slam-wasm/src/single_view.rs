use js_sys::Uint8ClampedArray;
use ndarray::prelude::*;
use nshare::ToNalgebra;
use slam_core::{point::Point2, single::SingleViewRecover};
use wasm_bindgen::prelude::*;
use web_sys::ImageData;

#[wasm_bindgen]
struct WrapperSingleViewRecover {
    points3d: Vec<f64>,
    colors: Vec<u8>,
}

#[wasm_bindgen]
impl WrapperSingleViewRecover {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WrapperSingleViewRecover {
        WrapperSingleViewRecover {
            points3d: vec![],
            colors: vec![],
        }
    }

    pub fn get_own_points3d(&self) -> Vec<f64> {
        self.points3d.iter().map(|val| *val).collect()
    }
    pub fn get_own_colors(&self) -> Vec<u8> {
        self.colors.iter().map(|val| *val).collect()
    }
    pub fn get_own_color_by_index(&self, index: usize) -> Vec<u8> {
        let offset = 4 * index;
        vec![
            self.colors[offset],
            self.colors[offset + 1],
            self.colors[offset + 2],
            self.colors[offset + 3],
        ]
    }
    /// 给定一张图片，并选取图像中不共面的3组平行线
    ///
    /// 将每条平行线的2个点按顺序放入到points中，
    ///
    /// points的长度为24
    pub fn single_view_recover(&mut self, image: ImageData, points: Vec<f64>) {
        // web_sys::console::log_1(&format!("{:?}", "single_view_recover").into());
        // web_sys::console::log_1(&format!("{:?}", points.len()).into());
        let data = &image.data();
        let height: usize = image.height() as usize;
        let width: usize = image.width() as usize;
        // web_sys::console::log_1(&format!("{:?}", height).into());
        // web_sys::console::log_1(&format!("{:?}", width).into());
        // web_sys::console::log_1(&format!("{:?}", data.len()).into());
        let data = ndarray::Array::from_shape_vec((height, width, 4usize), data.to_vec()).unwrap();
        if points.len() != 24 {
            panic!("the argument points len must be 12");
        }
        // web_sys::console::log_1(&format!("enumerate").into());
        let mut ps = vec![];
        for (index, val) in points.iter().enumerate() {
            if index % 2 == 1 {
                ps.push(Point2::new(points[index - 1], points[index]))
            }
        }

        let vp1 = SingleViewRecover::find_vanshing_point(&ps[0], &ps[1], &ps[2], &ps[3]);
        // web_sys::console::log_1(&format!("vp1 {:?}", vp1).into());
        let vp2 = SingleViewRecover::find_vanshing_point(&ps[4], &ps[5], &ps[6], &ps[7]);
        // web_sys::console::log_1(&format!("vp2 {:?}", vp2).into());
        let vp3 = SingleViewRecover::find_vanshing_point(&ps[8], &ps[9], &ps[10], &ps[11]);
        // web_sys::console::log_1(&format!("vp3 {:?}", vp3).into());
        let option_k =
            SingleViewRecover::compute_camera_params_from_vanshing_points(&vp1, &vp2, &vp3);
        if let Some(k) = option_k {
            // web_sys::console::log_1(&format!("k {:?}", k).into());
            let k_inv = k.into_nalgebra().try_inverse().unwrap().to_owned();
            let k_inv_raw_vec: Vec<f64> = k_inv.iter().map(|val| *val).collect();
            let k_inv = Array::from_shape_vec((3, 3), k_inv_raw_vec).unwrap();
            let mut points_3d: Vec<f64> = vec![];
            let mut colors: Vec<u8> = vec![];
            // struct recover
            for y in 0..height {
                for x in 0..width {
                    let coord_3d = k_inv.dot(&array![y as f64, x as f64, 1.]);
                    let color = data.slice(s![y, x, ..]);
                    points_3d.push(coord_3d[0]);
                    points_3d.push(coord_3d[1]);
                    points_3d.push(coord_3d[2]);
                    colors.push(color[0]);
                    colors.push(color[1]);
                    colors.push(color[2]);
                    colors.push(color[3]);
                }
            }
            self.points3d = points_3d;
            self.colors = colors;
        }
    }
}
