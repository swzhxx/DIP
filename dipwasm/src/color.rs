use std::f64::consts::PI;
use wasm_bindgen::prelude::*;
pub type NormalizeColorSpace = (f64, f64, f64);

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
    // #[wasm_bindgen(js_name = toRGB)]
    // pub fn to_rgb(&self) -> RGB {

    // }
}

use wasm_bindgen_test::*;

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
