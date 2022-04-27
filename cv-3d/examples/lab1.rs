use anyhow::anyhow;
use nalgebra::{
    AbstractRotation, Const, DMatrix, DVector, Dynamic, Matrix3, Matrix3x1, Matrix3x4, RowDVector,
    Vector2, Vector3, Vector4, SVD,
};
#[derive(Default)]
struct Camera {
    K: Matrix3<f64>,
    R: Matrix3<f64>,
    T: Matrix3x1<f64>,
}

impl Camera {
    fn calibrate(
        world_coor: &Vec<Vector3<f64>>,
        pixel_coor: &Vec<Vector2<f64>>,
    ) -> anyhow::Result<Camera> {
        if world_coor.len() != pixel_coor.len() {
            return Err(anyhow!("calibrate points must equal"));
        }
        if world_coor.len() < 6 || pixel_coor.len() < 6 {
            return Err(anyhow!("calibrate should have must > 6 points"));
        }
        let p = Camera::compose_p(world_coor, pixel_coor);
        println!("{}", p);
        let svd = p.svd(true, true);

        println!("singular_values values{}", svd.singular_values);
        let v_t = svd.v_t.unwrap();
        println!("v_t {}", v_t);

        // 因为nalgebra是列优先 所以需要先转为4x3矩阵，再进行一次转置
        let minest_v = v_t
            .transpose()
            .column(v_t.shape().1 - 1)
            .clone_owned()
            .reshape_generic(Dynamic::new(4), Dynamic::new(3))
            .transpose();

        println!("{}", minest_v);

        // let a = minest_v.slice((0, 0), (3, 3));
        // println!("a {}", a);
        // let b = minest_v.slice((0, 3), (3, 1));
        // println!("b {}", b);
        // todo!()

        let (K, R, T) = Camera::compute_k_rt(&minest_v);
        Ok(Camera { K, R, T })
    }
    fn compose_p(world_coor: &Vec<Vector3<f64>>, pixel_coor: &Vec<Vector2<f64>>) -> DMatrix<f64> {
        let point_num = world_coor.len();
        let mut p = DMatrix::from_diagonal_element(point_num * 2, 12, 1.);
        for row_index in 0..point_num {
            let mut w_row = world_coor[row_index].to_homogeneous();
            w_row.w = 1.;
            let mut p_row = pixel_coor[row_index].to_homogeneous().as_slice().to_vec();
            let zero = Vector4::from_element(0.);

            let px_coor = p_row[0];
            let up = -px_coor * w_row;
            let mut row = vec![];
            row.append(&mut w_row.as_slice().to_vec());
            row.append(&mut zero.as_slice().to_vec());
            row.append(&mut up.as_slice().to_vec());
            let row = &DVector::from_vec(row);
            p.row_mut(row_index * 2).copy_from(&row.transpose());

            let px_coor = p_row[1];
            let vp = -px_coor * w_row;
            let mut row = vec![];
            row.append(&mut zero.as_slice().to_vec());
            row.append(&mut w_row.as_slice().to_vec());
            row.append(&mut vp.as_slice().to_vec());
            p.row_mut(row_index * 2 + 1)
                .copy_from(&DVector::from_vec(row).transpose());
        }
        p
    }

    fn compute_k_rt(
        projection_matrix: &DMatrix<f64>,
    ) -> (Matrix3<f64>, Matrix3<f64>, Matrix3x1<f64>) {
        // let projection_matrix: Matrix3x4<f64> = Matrix3x4::new(
        //     5.09062054e-02,
        //     8.57319755e-05,
        //     1.13715227e-02,
        //     -0.80012034,
        //     -1.28406856e-03,
        //     5.23626831e-03,
        //     5.06416451e-02,
        //     -0.59539149,
        //     2.16381463e-05,
        //     4.64691136e-05,
        //     2.08447117e-05,
        //     0.00141669,
        // );

        let a = projection_matrix.slice((0, 0), (3, 3));
        let b = projection_matrix.slice((0, 3), (3, 1));
        print!("a {}", a);
        print!("b {}", b);
        // compute rho
        let a3_t = a.row(2);
        let rho = 1. / a3_t.norm();
        println!("rho {:?}", rho);
        let a1_t = a.row(0);
        let a2_t = a.row(1);
        // compute offset
        let cx = rho * rho * (a1_t.dot(&a3_t));
        let cy = rho * rho * (a2_t.dot(&a3_t));
        println!("cx :{} , cy :{}", cx, cy);
        // compute theta
        let a_cross13 = a1_t.cross(&a3_t);
        let a_cross23 = a2_t.cross(&a3_t);
        let theta: f64 =
            (-1. * (a_cross13.dot(&a_cross23) / (a_cross13.norm() * a_cross23.norm()))).acos();
        println!("theta {}", theta);

        //compute alpha and beta
        let alpha = rho * rho * (a_cross13.norm()) * theta.sin();
        let beta = rho * rho * (a_cross23.norm()) * theta.sin();
        println!("alpha : {}  ,beta :{}", alpha, beta);
        let K = Matrix3::new(
            alpha,
            -alpha * (1. / theta.tan()),
            cx,
            0.,
            beta / theta.sin(),
            cy,
            0.,
            0.,
            1.,
        );

        // compute R
        let r1 = &a_cross23 / a_cross23.norm();
        let r3 = rho * a3_t;
        let r2 = r3.cross(&r1);
        let mut R = Matrix3::from_element(0.);
        R.row_mut(0).copy_from(&r1);
        R.row_mut(1).copy_from(&r2);
        R.row_mut(2).copy_from(&r3);
        // compute T

        let T = rho * &K.try_inverse().unwrap() * &b;
        let T = Matrix3x1::new(T[0], T[1], T[2]);
        (K, R, T)
    }
}

fn main() {
    let mut w_xz = vec![
        Vector3::from_vec(vec![8., 0., 9.]),
        Vector3::from_vec(vec![8., 0., 1.]),
        Vector3::from_vec(vec![6., 0., 1.]),
        Vector3::from_vec(vec![6., 0., 9.]),
    ];
    let mut w_xy = vec![
        Vector3::from_vec(vec![5., 1., 0.]),
        Vector3::from_vec(vec![5., 9., 0.]),
        // Vector3::from_vec(vec![4., 9., 0.]),
        // Vector3::from_vec(vec![4., 1., 0.]),
    ];
    let mut w_yz = vec![
        // Vector3::from_vec(vec![0., 4., 7.]),
        // Vector3::from_vec(vec![0., 4., 3.]),
        // Vector3::from_vec(vec![0., 8., 3.]),
        // Vector3::from_vec(vec![0., 8., 7.]),
    ];

    let mut w_coor = vec![];
    w_coor.append(&mut w_xz);
    w_coor.append(&mut w_xy);
    w_coor.append(&mut w_yz);

    let mut c_xz = vec![
        Vector2::from_vec(vec![275., 142.]),
        Vector2::from_vec(vec![312., 454.]),
        Vector2::from_vec(vec![382., 436.]),
        Vector2::from_vec(vec![357., 134.]),
    ];

    let mut c_xy = vec![
        Vector2::from_vec(vec![432., 473.]),
        Vector2::from_vec(vec![612., 623.]),
        // Vector2::from_vec(vec![647., 606.]),
        // Vector2::from_vec(vec![464., 465.]),
    ];

    let mut c_yz = vec![
        // Vector2::from_vec(vec![654., 216.]),
        // Vector2::from_vec(vec![644., 368.]),
        // Vector2::from_vec(vec![761., 420.]),
        // Vector2::from_vec(vec![781., 246.]),
    ];

    let mut c_coor = vec![];
    c_coor.append(&mut c_xz);
    c_coor.append(&mut c_xy);
    c_coor.append(&mut c_yz);

    let w_check = vec![
        Vector4::from_vec(vec![6., 0., 5., 1.]),
        Vector4::from_vec(vec![3., 3., 0., 1.]),
        Vector4::from_vec(vec![0., 4., 0., 1.]),
        Vector4::from_vec(vec![0., 4., 4., 1.]),
        Vector4::from_vec(vec![0., 0., 7., 1.]),
    ];

    let c_check = vec![
        Vector2::from_vec(vec![369., 297.]),
        Vector2::from_vec(vec![531., 484.]),
        Vector2::from_vec(vec![640., 468.]),
        Vector2::from_vec(vec![646., 333.]),
        Vector2::from_vec(vec![556., 194.]),
    ];

    let camera = Camera::calibrate(&w_coor, &c_coor).unwrap();
    println!("k {}", camera.K);
    println!("R {}", camera.R);
    println!("T {}", camera.T);
}
