use nalgebra::DMatrix;

pub fn compute_min_vt_eigen_vector(m: &DMatrix<f64>) -> Vec<f64> {
    let v = m.transpose() * m;
    let svd = v.svd(true, false);
    let (ix, &v) = svd
        .singular_values
        .iter()
        .enumerate()
        .min_by_key(|&(_, &v)| float_ord::FloatOrd(v))
        .unwrap();
    let min_eigen_vector: Vec<f64> = svd.u.unwrap().column(ix).into_iter().map(|v| *v).collect();
    min_eigen_vector
}

#[cfg(test)]
mod test {
    #[test]
    fn test_svd() {}
}
