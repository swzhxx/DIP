use nalgebra::{
    allocator::Allocator, ComplexField, DMatrix, DefaultAllocator, Dim, DimDiff, DimMin,
    DimMinimum, DimSub, SVD, U1,
};

pub fn compute_min_vt_eigen_vector(m: &DMatrix<f64>) -> Vec<f64> {
    let v = m.transpose() * m;
    let svd = v.svd(true, false);
    let (ix, &v) = svd
        .singular_values
        .iter()
        .enumerate()
        .min_by_key(|&(_, &v)| float_ord::FloatOrd(v))
        .unwrap();
    let u = svd.u.unwrap();
    let column = u.column(ix);
    let min_eigen_vector: Vec<f64> = column.into_iter().map(|v| *v).collect();
    min_eigen_vector
}

pub fn sort_svd<T, R, C>(svd: &mut SVD<T, R, C>)
where
    T: ComplexField,
    R: DimMin<C>,
    C: Dim,
{
    todo!()
}

#[cfg(test)]
mod test {
    use super::*;
    use nalgebra::{Const, Dynamic, Matrix3x2};
    #[test]
    fn test_compute_min_vt_eigen_vector() {
        let a = DMatrix::from_vec(3, 2, vec![0., 1., 1., 1., 1., 0.]);

        let eigen_vector = compute_min_vt_eigen_vector(&a);
        println!("minize eigen vector : {:?}", eigen_vector);

        let b = DMatrix::from_vec(
            3,
            4,
            vec![
                -7753925.999447657,
                3252.0580716000004,
                4819.286752846649,
                1.0,
                -2769430.4285714286,
                1211.2142857142858,
                -1760.8571428571436,
                1.0,
                13796142.333698358,
                7664.272357314287,
                -1760.8571428571436,
                1.0,
            ],
        );

        let eigen_vector = compute_min_vt_eigen_vector(&b);
        println!("minize eigen vector : {:?}", eigen_vector);
    }
}
