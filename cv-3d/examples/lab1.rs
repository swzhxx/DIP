use nalgebra::{Vector2, Vector3, Vector4};

fn main() {
    let mut w_xz = vec![
        Vector3::from_vec(vec![8, 0, 9]),
        Vector3::from_vec(vec![8, 0, 1]),
        Vector3::from_vec(vec![6, 0, 1]),
        Vector3::from_vec(vec![6, 0, 9]),
    ];
    let mut w_xy = vec![
        Vector3::from_vec(vec![5, 1, 0]),
        Vector3::from_vec(vec![5, 9, 0]),
        Vector3::from_vec(vec![4, 9, 0]),
        Vector3::from_vec(vec![4, 1, 0]),
    ];
    let mut w_yz = vec![
        Vector3::from_vec(vec![0, 4, 7]),
        Vector3::from_vec(vec![0, 4, 3]),
        Vector3::from_vec(vec![0, 8, 3]),
        Vector3::from_vec(vec![0, 8, 7]),
    ];

    let mut w_coor = vec![];
    w_coor.append(&mut w_xz);
    w_coor.append(&mut w_xy);
    w_coor.append(&mut w_yz);

    let mut c_xz = vec![
        Vector2::from_vec(vec![275, 142]),
        Vector2::from_vec(vec![312, 454]),
        Vector2::from_vec(vec![382, 436]),
        Vector2::from_vec(vec![357, 134]),
    ];

    let mut c_xy = vec![
        Vector2::from_vec(vec![432, 473]),
        Vector2::from_vec(vec![612, 623]),
        Vector2::from_vec(vec![647, 606]),
        Vector2::from_vec(vec![464, 465]),
    ];

    let mut c_yz = vec![
        Vector2::from_vec(vec![654, 216]),
        Vector2::from_vec(vec![644, 368]),
        Vector2::from_vec(vec![761, 420]),
        Vector2::from_vec(vec![781, 246]),
    ];

    let mut c_coor = vec![];
    c_coor.append(&mut c_xz);
    c_coor.append(&mut c_xy);
    c_coor.append(&mut c_yz);

    let w_check = vec![
        Vector4::from_vec(vec![6, 0, 5, 1]),
        Vector4::from_vec(vec![3, 3, 0, 1]),
        Vector4::from_vec(vec![0, 4, 0, 1]),
        Vector4::from_vec(vec![0, 4, 4, 1]),
        Vector4::from_vec(vec![0, 0, 7, 1]),
    ];

    let c_check = vec![
        Vector2::from_vec(vec![369, 297]),
        Vector2::from_vec(vec![531, 484]),
        Vector2::from_vec(vec![640, 468]),
        Vector2::from_vec(vec![646, 333]),
        Vector2::from_vec(vec![556, 194]),
    ];
}
