use sprs::{CsMat, CsMatBase, TriMat};

fn main() {
    let mut a = TriMat::new((4, 4));
    a.add_triplet(0, 0, 3.0_f64);
    a.add_triplet(1, 2, 2.0);
    a.add_triplet(3, 0, -2.0);

    // 这个矩阵类型不允许进行计算，需要
    // 转换为兼容的稀疏矩阵类型，例如
    let b: CsMatBase<f64, usize, Vec<usize>, Vec<usize>, Vec<f64>> = a.to_csr();

    // let eye = CsMat::eye(3);
    // let a = CsMat::new_csc(
    //     (3, 3),
    //     vec![0, 2, 4, 5],
    //     vec![0, 1, 0, 2, 2],
    //     vec![1., 2., 3., 4., 5.],
    // );
    // let b = &eye * &a;
    // assert_eq!(a, b.to_csr());
}
