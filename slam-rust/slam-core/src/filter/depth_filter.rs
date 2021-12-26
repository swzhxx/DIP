use std::{f64::consts::PI, marker::PhantomData, thread::current};

use nalgebra::{
    matrix, vector, AbstractRotation, Const, DMatrix, Dynamic, Matrix, Matrix2, Matrix3, Matrix3x2,
    Matrix3x4, MatrixXx3, Vector2, Vector3, Vector4,
};
use ndarray::{array, s, Array, Array1, Array2, ArrayBase, Axis, OwnedRepr};
use nshare::{RefNdarray2, ToNalgebra};

use crate::{
    point::Point3,
    triangulate::{self, RelativeDltTriangulator, Triangulate},
    utils::{cam2px, ncc, px2cam},
};

/// (u,v,x,y,z)
pub type Pixel3dCoordinate = (f64, f64, f64, f64, f64);

const border: usize = 20;

pub struct DepthFilter<'a> {
    images: &'a Vec<Array2<f64>>,
    height: usize,
    width: usize,
    pub depth_matrix: Array2<f64>,
    depth_cov2_matrix: Array2<f64>,
    min_depth: f64,
    max_depth: f64,
    reader: Box<dyn Fn(&'a Vec<Array2<f64>>) -> ReaderResult<'a>>,
    // ref_image: Option<&'a Array2<f64>>,
    // current_image: Option<&'a Array2<f64>>, // images: Vec<Array2<f64>>,
    pub pixel_3d_coordinate: Vec<Pixel3dCoordinate>,
    k1: &'a Array2<f64>,
    k2: &'a Array2<f64>,
}

impl<'a> DepthFilter<'a> {
    /// 极线搜索
    /// @return (匹配点坐标， 极线方向)
    fn epipolar_search(
        &self,
        pt_ref: &Array1<f64>,
        ref_image: &Array2<f64>,
        current_image: &Array2<f64>,
        pose: &Array2<f64>,
        depth: f64,
        depth_cov: f64,
    ) -> Option<(Vector2<f64>, Vector2<f64>)> {
        let mut k2 = self.k2.clone();
        let mut a = (k2.dot(pose));
        a.push(Axis(0), array![0., 0., 0., 1.].view());
        let projection = a.into_nalgebra();

        let f_ref = px2cam(&vector![pt_ref[0], pt_ref[1]], self.k1);
        let f_ref = f_ref.normalize();

        let mut P_ref = (&f_ref * depth).to_homogeneous();
        P_ref[3] = 1.;
        let mut d_min = depth - 3. * depth_cov;
        let d_max = depth + 3. * depth_cov;
        if d_min < 0.1 {
            d_min = 0.1
        }
        let mut min_P_ref = (&f_ref * d_min).to_homogeneous();
        min_P_ref[3] = 1.;
        let mut max_P_ref = (&f_ref * d_max).to_homogeneous();
        max_P_ref[3] = 1.;

        let P_ref_rt = P_ref.clone_owned();
        // let px_mean_curr = cam2px(&P_ref_rt.xyz(), &self.camera);
        let px_mean_curr = Vector4::from_vec((&projection * P_ref).data.as_vec().to_vec());
        let px_mean_curr = (px_mean_curr / px_mean_curr.z).xy();
        let min_p_curr = Vector4::from_vec((&projection * min_P_ref).data.as_vec().to_vec());
        let px_min_curr = (min_p_curr / min_p_curr.z).xy();
        let max_p_curr = Vector4::from_vec((&projection * max_P_ref).data.as_vec().to_vec());
        let px_max_curr = (max_p_curr / max_p_curr.z).xy();

        // let px_min_curr = cam2px(
        //     &vector![min_f_ref_rt[0], min_f_ref_rt[1], min_f_ref_rt[2]],
        //     &self.camera,
        // );
        // let px_max_curr = cam2px(
        //     &vector![max_f_ref_rt[0], max_f_ref_rt[1], max_f_ref_rt[2]],
        //     &self.camera,
        // );
        let epipolar_line = px_max_curr - px_min_curr;
        let epipolar_direction = epipolar_line.normalize();
        let mut half_length = 0.5 * epipolar_line.norm();

        if half_length > 100. {
            half_length = 100.;
        }
        let mut best_ncc = -1.;
        let mut best_px_curr = vector![0., 0.];
        let mut l = -half_length;
        let v = vector![pt_ref[0], pt_ref[1]];
        if !inside(&v, self.width, self.height) {
            return None;
        }
        while l <= half_length {
            let px_curr = px_mean_curr + l * epipolar_direction;
            if !inside(&px_curr, self.width, self.height) {
                l = l + 0.7;
                continue;
            }
            let ncc_value = ncc(
                ref_image,
                current_image,
                (pt_ref[0], pt_ref[1]),
                (px_curr.x, px_curr.y),
                Some(3),
            );
            if ncc_value > best_ncc {
                best_ncc = ncc_value;
                best_px_curr = px_curr;
            }
            l = l + 0.7;
        }
        if best_ncc < 0.8 {
            None
        } else {
            Some((best_px_curr, epipolar_direction))
        }
    }

    pub fn excute(&mut self) {
        let (ref_image, option_image, option_pose) = self.reader();

        if ref_image != None && option_image != None && option_pose != None {
            self.update(
                ref_image.unwrap(),
                option_image.unwrap(),
                &option_pose.unwrap(),
            );
            self.excute();
        }
    }

    pub fn new(
        images: &'a Vec<Array2<f64>>,
        height: usize,
        width: usize,
        depth_mean: Option<f64>,
        depth_cov: Option<f64>,
        min_depth: Option<f64>,
        max_depth: Option<f64>,
        reader: Box<dyn Fn(&'a Vec<Array2<f64>>) -> ReaderResult<'a>>,
        k1: &'a Array2<f64>,
        k2: &'a Array2<f64>,
    ) -> Self {
        let depth_mean = match depth_mean {
            Some(val) => val,
            _ => 3.,
        };
        let depth_cov = match depth_cov {
            Some(val) => val,
            _ => 20.,
        };
        let min_depth = match min_depth {
            Some(val) => val,
            _ => 0.1,
        };
        let max_depth = match max_depth {
            Some(val) => val,
            _ => 100.,
        };
        DepthFilter {
            images,
            height,
            width,
            depth_matrix: Array::from_elem((height, width), depth_mean),
            depth_cov2_matrix: Array::from_elem((height, width), depth_cov),
            min_depth,
            max_depth,
            reader, // images,
            k1,
            k2,

            pixel_3d_coordinate: vec![],
        }
    }

    fn reader(&self) -> ReaderResult<'a> {
        let reader = &self.reader;
        reader(&self.images)
    }

    /// 对整个深度图更新
    pub fn update(
        &mut self,
        ref_image: &Array2<f64>,
        current_image: &Array2<f64>,
        pose: &Array2<f64>,
    ) {
        let mut pose = pose.clone().to_owned();
        // pose.push(Axis(0), array![0., 0., 0., 1.].view())
        //     .expect(&format!("pose {:?}", pose));
        let mut update_count = 0;

        for y in border..(self.height - border) {
            for x in border..(self.width - border) {
                if self.depth_cov2_matrix[[y, x]] < self.min_depth
                    || self.depth_cov2_matrix[[y, x]] > self.max_depth
                {
                    continue;
                }
                let pt_ref = array![x as f64, y as f64];
                let depth = self.depth_matrix[[y, x]];
                let depth_cov = self.depth_cov2_matrix[[y, x]];
                if let Some((pt_curr, epiploar_direction)) =
                    self.epipolar_search(&pt_ref, ref_image, current_image, &pose, depth, depth_cov)
                {
                    // // TODO: 更新深度图
                    // self.update_depth_filter(
                    //     &vector![x as f64, y as f64],
                    //     &pt_curr,
                    //     &projection,
                    //     &epiploar_direction,
                    // );

                    // let mut ref_data = pt_ref.to_owned().into_raw_vec();
                    // // ref_data.push(1.);
                    // let pt_ref = Vector2::from_vec(ref_data);
                    // // let pt_curr = pt_curr;
                    // let projection = Matrix3x4::from_vec(projection.to_owned().into_raw_vec());
                    // let P = triangulate
                    //     .triangulate_relative(&projection, &pt_ref, &pt_curr)
                    //     .unwrap();
                    self.update_depth_rlt(&pt_ref, &pose, &pt_curr);
                    update_count = update_count + 1;
                    if update_count % 100 == 0 {
                        println!("update count ing {:?}", update_count);
                    }
                }
            }
        }
        println!("update count {:?}", update_count);
    }

    /// 深度图更新
    /// 通过三角化， 进行深度图更新
    fn update_depth_filter(
        &mut self,
        pt_ref: &Vector2<f64>,
        pt_curr: &Vector2<f64>,
        projection: &Array2<f64>,
        epipolar_direction: &Vector2<f64>,
    ) {
        let projection = projection.clone().into_nalgebra();
        // let f_ref = pt_ref.to_homogeneous();
        let f_ref = pt_ref.to_homogeneous().normalize();
        let f_curr = pt_curr.to_homogeneous();
        let f_curr = f_curr.normalize();

        let R = projection.slice((0, 0), (3, 3));
        let R: DMatrix<f64> = R.try_inverse().unwrap();
        let t = vector![projection[(0, 3)], projection[(1, 3)], projection[(2, 3)]];

        let f2 = R * f_curr;
        let b = vector![f_ref.dot(&t), t.dot(&f2)];
        let A: Matrix2<f64> = Matrix2::new(
            f_ref.dot(&f_ref),
            -f_ref.dot(&f2),
            f2.dot(&f_ref),
            -f2.dot(&f2),
        );
        let ans = A.try_inverse().unwrap() * b;

        let xm = ans[0] * f_ref;
        let xn = ans[1] * f2 + t;
        let p_esti = (xm + xn) / 2.;
        let dept_estimation = p_esti.norm();
        // println!("dept_estimation {:?}", dept_estimation);
        // self.depth_matrix[(pt_ref.y as usize, pt_ref.x as usize)] = dept_estimation;

        // 计算不确定性
        let p = f_ref * dept_estimation;
        let a = p - (t);
        let t_norm = t.norm();
        let a_norm = a.norm();
        let alpha = (f_ref.dot(&t) / t_norm).acos();
        // let beta = (-a.dot(&t) / (a_norm * t_norm)).acos();

        let f_curr_prim = (pt_curr + epipolar_direction).to_homogeneous();

        let f_curr_prim = f_curr_prim.normalize();
        let beta_prim = f_curr_prim.dot(&-t) / t_norm;
        let gamma = PI - alpha - beta_prim;
        let p_prim = t_norm * beta_prim.sin() / gamma.sin();
        let d_cov = p_prim - dept_estimation;
        let d_cov2 = d_cov * d_cov;

        //高斯融合
        let mu = self
            .depth_matrix
            .get((pt_ref.y as usize, pt_ref.x as usize))
            .unwrap();
        let sigma2 = self
            .depth_cov2_matrix
            .get((pt_ref.y as usize, pt_ref.x as usize))
            .unwrap();

        let mu_fuse = d_cov2 * mu + sigma2 * dept_estimation;
        let sigma_fuse2 = (sigma2 * d_cov2) / (sigma2 + d_cov2);
        // println!("mu_fuse {:?}", mu_fuse);
        self.depth_matrix[(pt_ref.y as usize, pt_ref.x as usize)] = mu_fuse;
        self.depth_cov2_matrix[(pt_ref.y as usize, pt_ref.x as usize)] = sigma_fuse2;
    }
    fn update_depth_rlt(
        &mut self,
        pt_ref: &Array1<f64>,
        pose: &Array2<f64>,
        pt_curr: &Vector2<f64>,
    ) {
        let triangulate = RelativeDltTriangulator::new();
        let ref_data = pt_ref.to_owned().into_raw_vec();
        // ref_data.push(1.);
        let pt_ref = Vector2::from_vec(ref_data);
        // let pt_curr = pt_curr;
        let k2 = self.k2.clone();
        let mut a = (k2.dot(pose));
        a.push(Axis(0), array![0., 0., 0., 1.].view());
        let p2 = a.into_nalgebra();
        let p2 = p2.slice((0, 0), (3, 4)).into_owned();
        let p2 = Matrix3x4::from_vec(p2.data.as_vec().to_vec());
        let mut p1 = self.k1.to_owned();
        p1.push_column(array![0., 0., 0.].view());
        let p1 = p1.into_nalgebra();
        let p1 = Matrix3x4::from_vec(p1.data.as_vec().to_vec());

        let P = triangulate
            .triangulate_relative(&p1, &p2, &pt_ref, &pt_curr)
            .unwrap();
        if P.z < 0. {
            return;
        }
        self.pixel_3d_coordinate
            .push((pt_ref.x, pt_ref.y, P.x, P.y, P.z));

        // let real = p1 * P.to_homogeneous();
        // let reala = real / real.z;

        // let realb = p2 * P.to_homogeneous();
        // let realb = realb / realb.z;
        // println!("image1 a{:?}", reala);
        // println!("image1 b{:?}", realb);
    }
}

/// 0：ref图像数据 1.curr图像数据 ， 2.projection矩阵
pub type ReaderResult<'b> = (
    Option<&'b Array2<f64>>,
    Option<&'b Array2<f64>>,
    Option<Array2<f64>>,
);

fn inside(pt: &Vector2<f64>, width: usize, height: usize) -> bool {
    let _border: f64 = 20.;
    let x = pt[(0, 0)];
    let y = pt[(1, 0)];
    return x >= _border as f64
        && y >= _border as f64
        && (x + _border) < width as f64
        && (y + _border) < height as f64;
}

#[cfg(test)]
mod test {
    use std::cell::RefCell;

    use image::{self, DynamicImage, GenericImageView};
    use ndarray::{array, Array, Array2};
    use nshare::RefNdarray2;

    use crate::{
        features::fast::OFast,
        filter::depth_filter::ReaderResult,
        matches::orb::Orb,
        sfm::{essential_decomposition, get_projection_through_fundamental, EightPoint},
    };

    use super::DepthFilter;
    #[test]
    fn test_depth_filter() {
        let image_1 = image::open("D:/dip/DIP/slam-rust/slam-core/tests/images/1.png")
            .unwrap()
            .grayscale();
        let image_2 = image::open("D:/dip/DIP/slam-rust/slam-core/tests/images/2.png")
            .unwrap()
            .grayscale();
        let (width, height) = image_1.dimensions();

        let image_to_ndarray = |img: DynamicImage| {
            let mut img_array = Array::from_elem((height as usize, width as usize), 0.);
            for y in 0..height {
                for x in 0..width {
                    let pixel = img.get_pixel(x, y).0.clone();
                    let gray = &pixel[0];
                    img_array[[y as usize, x as usize]] = *gray as f64
                }
            }
            img_array
        };
        let image_1_nd = image_to_ndarray(image_1);
        let image_2_nd = image_to_ndarray(image_2);

        let images = vec![image_1_nd, image_2_nd];
        let mut i = RefCell::new(0);

        // let ref_features = OFast::new(&images[0]).find_features(Some(40.));
        // let ref_descriptors = Orb::new(&images[0], &ref_features).create_descriptors();
        let k = array![[520.9, 0., 325.1], [0., 521., 249.7], [0., 0., 1.]];
        let k_clone = k.clone();
        let reader: Box<dyn for<'a> Fn(&'a Vec<Array2<f64>>) -> ReaderResult<'a>> =
            Box::new(move |images| {
                let _i = *i.borrow();
                if _i > 1 {
                    return (None, None, None);
                }
                let ref_image = &images[0];
                let curr_image = &images[1];

                // let curr_features = OFast::new(&images[1]).find_features(Some(40.));
                // let curr_descriptors = Orb::new(&images[1], &curr_features).create_descriptors();
                // let matches = Orb::brief_match(&ref_descriptors, &curr_descriptors, 40);
                // let matches1 = matches
                //     .iter()
                //     .map(|dmatch| ref_features[dmatch.i1].clone().f())
                //     .collect();
                // let matches2 = matches
                //     .iter()
                //     .map(|dmatch| curr_features[dmatch.i2].clone().f())
                //     .collect();
                // let mut eight_point = EightPoint::new(&matches1, &matches2);
                // let esstinal = eight_point.normalize_find_esstinal().unwrap();
                // println!("esstinal... {:?}", esstinal);

                // let fundamental = eight_point.normalize_find_fundamental().unwrap();

                // println!("fundamental... {:?}", fundamental);
                // let pose = essential_decomposition(&esstinal);
                // println!("pose {:?}", pose);
                // let esstinal = (&k_clone).t().dot(&esstinal).dot(&k_clone);
                // let pose = essential_decomposition(&esstinal);
                // println!("esstinal 2 {:?}", esstinal);
                // println!("pose2 {:?}", pose);
                // let fundamental = array![
                //     [
                //         4.544437503937326e-6,
                //         0.0001333855576988952,
                //         -0.01798499246457619
                //     ],
                //     [
                //         -0.0001275657012959839,
                //         2.266794804637672e-5,
                //         -0.01416678429259694
                //     ],
                //     [0.01814994639952877, 0.004146055871509035, 1.]
                // ];

                // todo!();
                // let projection = get_projection_through_fundamental(&fundamental);

                // let pose = pose.ref_ndarray2().to_owned();
                let pose = array![
                    [
                        0.9969387384756405,
                        -0.0515557418857258,
                        0.05878058527448649,
                        -0.935080288539632
                    ],
                    [
                        0.05000441581116598,
                        0.9983685317362444,
                        0.02756507279509838,
                        -0.03514646277098749
                    ],
                    [
                        -0.06010582439317147,
                        -0.02454140007064545,
                        0.9978902793176159,
                        0.352689070059345
                    ]
                ];
                *i.borrow_mut() = _i + 1;
                (Some(ref_image), Some(curr_image), Some(pose))
            });
        let mut depth_filter = DepthFilter::new(
            &images,
            height as usize,
            width as usize,
            None,
            None,
            None,
            None,
            reader,
            &k,
            &k,
        );
        depth_filter.excute();
        // println!("depth_matrix {:?}", depth_filter.depth_matrix);
    }
}
