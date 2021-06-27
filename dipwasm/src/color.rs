use ndarray::{Array3, ArrayBase};
use num_traits::{FromPrimitive, Num, NumAssign, NumAssignOps, ToPrimitive, Zero};
use std::{
    f64::consts::{FRAC_PI_3, PI},
    ops::{Div, Sub},
};
use wasm_bindgen::prelude::*;
pub type NormalizeColorSpace = (f64, f64, f64);
use ndarray_stats::QuantileExt;

#[wasm_bindgen]
pub struct RGB(u8, u8, u8);

#[wasm_bindgen]
impl RGB {
    #[wasm_bindgen(constructor)]
    pub fn new(r: u8, g: u8, b: u8) -> RGB {
        RGB(r, g, b)
    }

    #[wasm_bindgen(js_name = toHSI)]
    pub fn to_hsi(&self) -> HSI {
        let (r, g, b) = self.normalize();
        let theta = (0.5 * ((r - g) + (r - b))
            / (((r - g).powi(2) + (r - b) * (g - b)).powf(0.5) + 0.00000001))
            .acos();
        let theta = theta * 180. / PI;

        let h = if b <= g { theta } else { 360.0 - theta };
        let h = h / 360.;

        let min_r_g_b = r.min(g.min(b));
        let s = if min_r_g_b == 0. {
            1.
        } else {
            1. - (3. / (r + g + b)) * min_r_g_b
        };
        let i = (1. / 3.) * (r + g + b);
        HSI::new(h, s, i)
    }

    fn normalize(&self) -> NormalizeColorSpace {
        let r = self.0 as f64;
        let g = self.1 as f64;
        let b = self.2 as f64;
        // let total = 255. * 3.;
        (r / 255., g / 255., b / 255.)
    }
}

#[wasm_bindgen]
pub struct HSI(f64, f64, f64);

#[wasm_bindgen]
impl HSI {
    #[wasm_bindgen(constructor)]
    pub fn new(h: f64, s: f64, i: f64) -> HSI {
        HSI(h, s, i)
    }
    #[wasm_bindgen(js_name = toRGB)]
    pub fn to_rgb(&self) -> RGB {
        let HSI(temp_h, temp_s, temp_i) = self;
        let temp_h = temp_h * 360. % 360.;

        let temp_0 = temp_i * (1. - temp_s);
        let temp_1 = temp_i
            * (1.
                + temp_s * (temp_h / 180. * PI).cos()
                    / ((FRAC_PI_3 - temp_h / 180. * PI).cos() + 0.0000001));
        let temp_2 = 3. * temp_i - (temp_0 + temp_1);

        let mut r: f64;
        let mut g: f64;
        let mut b: f64;

        if 0. <= temp_h && temp_h <= 120. {
            b = temp_0;
            r = temp_1;
            g = temp_2;
        } else if 120. <= temp_h && temp_h < 240. {
            r = temp_0;
            g = temp_1;
            b = temp_2;
        } else {
            g = temp_0;
            b = temp_1;
            r = temp_2;
        }

        let r: u8 = (r * 255.).round() as u8;
        let g: u8 = (g * 255.).round() as u8;
        let b: u8 = (b * 255.).round() as u8;
        RGB(r, g, b)
    }
}

pub fn normalize_color(v: &Array3<f64>) -> Array3<u8> {
    unsafe {
        web_sys::console::log_1(&format!("normalize_color_a").into());
        web_sys::console::log_1(&format!("normalize_color_a.{:?}",v.max_skipnan()).into());
    }
    let max = v.max_skipnan() + 0.0000000001;
    unsafe {
        web_sys::console::log_1(&format!("normalize_color_b").into());
    }
    let min = v.min_skipnan();
    unsafe {
        web_sys::console::log_1(&format!("normalize_color_c").into());
    }
    let mean = v.mean().unwrap();
    unsafe {
        web_sys::console::log_1(&format!("normalize_color_d").into());
    }
    let offset = if 0. - *min > 0. { 0. - *min } else { 0. };
    unsafe {
        web_sys::console::log_1(&format!("normalize_color").into());
    }
    v.map(|v| (((*v + offset) / max) * 255.) as u8)
}

#[cfg(test)]
mod test {
    use super::normalize_color;
    use super::*;
    use ndarray::prelude::*;
    use wasm_bindgen_test::*;
    #[wasm_bindgen_test]
    fn test_normalize_color() {
        let a = array![[[1.], [1.], [1.]], [[1.], [1.], [1.]], [[1.], [1.], [1.]]];
        assert_eq!(
            normalize_color(&a),
            array![
                [[254], [254], [254]],
                [[254], [254], [254]],
                [[254], [254], [254]]
            ]
        )
    }

    #[wasm_bindgen_test]
    fn rgb_to_hsi() {
        let rgb = RGB::new(255, 0, 0);
        let hsi: HSI = rgb.to_hsi();
        let difference_h = (hsi.0 - 0.).abs();
        let difference_s = (hsi.1 - 1.).abs();
        let difference_i = (hsi.2 - 0.33333).abs();

        assert!(difference_h < 1e-3);
        assert!(difference_s < 1e-3);
        assert!(difference_i < 1e-3);

        let rgb = RGB::new(255, 255, 0);
        let hsi: HSI = rgb.to_hsi();
        let difference_h = (hsi.0 - 60. / 360.).abs();
        let difference_s = (hsi.1 - 1.).abs();
        let difference_i = (hsi.2 - 0.66667).abs();

        assert!(difference_h < 1e-3);
        assert!(difference_s < 1e-3);
        assert!(difference_i < 1e-3);

        let rgb = RGB::new(255, 255, 255);
        let hsi: HSI = rgb.to_hsi();
        let difference_h = (hsi.0 - 0.25).abs();
        let difference_s = (hsi.1 - 0.).abs();
        let difference_i = (hsi.2 - 1.).abs();

        assert!(difference_h < 1e-3);
        assert!(difference_s < 1e-3);
        assert!(difference_i < 1e-3);
    }

    #[wasm_bindgen_test]
    fn his_to_rgb() {
        let hsi = HSI::new(0., 1., 0.333333337);

        let rgb = hsi.to_rgb();
        assert_eq!(rgb.0, 255);
        assert_eq!(rgb.1, 0);
        assert_eq!(rgb.2, 0);

        let hsi = HSI::new(1. / 6., 1., 0.66666666667);
        let rgb = hsi.to_rgb();
        assert_eq!(rgb.0, 255);
        assert_eq!(rgb.1, 255);
        assert_eq!(rgb.2, 0);

        let hsi = HSI::new(0., 0., 1.);
        let rgb = hsi.to_rgb();
        assert_eq!(rgb.0, 255);
        assert_eq!(rgb.1, 255);
        assert_eq!(rgb.2, 255);
    }
}
