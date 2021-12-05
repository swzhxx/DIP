use nalgebra::{
    allocator::Allocator, storage::Storage, ComplexField, DMatrix, DefaultAllocator, Dim, DimDiff,
    DimMin, DimMinimum, DimName, DimSub, Matrix1, Matrix1xX, MatrixSlice1xX, OVector, VecStorage,
    VectorN, SVD, U1,
};
use nshare::RefNdarray2;

pub fn compute_min_vt_eigen_vector(m: &DMatrix<f64>) -> Vec<f64> {
    let v = m.transpose() * m;
    let svd = v.svd(true, false);
    let (ix, &v) = svd
        .singular_values
        .iter()
        .enumerate()
        .min_by_key(|&(_, &v)| float_ord::FloatOrd(v))
        .expect("single_values failed");
    let u = svd.u.expect("u failed");
    let column = u.column(ix).to_owned();
    let min_eigen_vector: Vec<f64> = column.ref_ndarray2().to_owned().into_raw_vec();
    min_eigen_vector
}

pub fn sort_svd<T, R, C>(svd: &mut SVD<T, R, C>)
where
    T: ComplexField,
    R: DimMin<C>,
    C: Dim,
    DefaultAllocator: Allocator<T, DimMinimum<R, C>, C>
        + Allocator<T, R, DimMinimum<R, C>>
        + Allocator<T::RealField, DimMinimum<R, C>>,
{
    let mut s: Vec<(_, _)> = svd
        .singular_values
        .iter()
        .enumerate()
        .map(|(idx, v)| (v, idx))
        .collect();
    s.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    let order: Vec<usize> = s.iter().map(|t| t.1).collect();
    let single_values: Vec<_> = s.iter().map(|t| t.0.clone()).collect();

    // let u_clone = svd.u.as_ref().to_owned().unwrap().clone().to_owned();
    // let v_t_clone = svd.v_t.as_ref().to_owned().unwrap().clone().to_owned();
    let u_clone = svd.u.as_ref().unwrap().clone_owned();
    let v_t_clone = svd.v_t.as_ref().unwrap().clone_owned();
    let u_ref = svd.u.as_mut().unwrap();
    let v_t_ref = svd.v_t.as_mut().unwrap();
    for (index, i) in order.iter().enumerate() {
        // u_ref.set_column(*i, &u_clone.column(*i));
        // v_t_ref.set_row(*i, &v_t_clone.row(*i));
        u_ref.column_mut(index).copy_from(&u_clone.column(*i));
        v_t_ref.row_mut(index).copy_from(&v_t_clone.row(*i));
        // svd.singular_values.set_row(single_values);
        let single_value = single_values[index].clone();
        svd.singular_values[index] = single_value;
    }
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
