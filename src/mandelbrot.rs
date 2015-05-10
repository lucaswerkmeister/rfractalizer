extern crate gdk;

use complex::Complex;
use std::sync::mpsc::Receiver;

fn iterate(c: Complex, max_iterations: i64) -> i64 {
    let mut z = Complex { r: 0.0, i: 0.0 };
    for i in 0..max_iterations {
        if z.mag_sqr() >= 4.0 {
            return i;
        }
        z = z*z + c;
    }
    return -1;
}

// assumes RGB, 3 channels, rowstride = 3*width (i. e.: no alpha channel!)
pub fn draw<ColorPalette>(neg_corner: Complex, pos_corner: Complex, max_iterations: i64, pixels: &mut [u8], width: i32, height: i32, supersampling: i32, color_palette: ColorPalette, cancel_rx: Receiver<bool>)
    where ColorPalette : Fn(i64) -> (i32,i32,i32) {
    for y in 0..height {
        for x in 0..width {
            let (mut r, mut g, mut b) = (0,0,0);
            for dy in 0..supersampling {
                for dx in 0..supersampling {
                    let xf = (x as f64 + (dx as f64 / supersampling as f64)) / width as f64;
                    let yf = (y as f64 + (dy as f64 / supersampling as f64)) / height as f64;
                    let c = Complex {
                        r: neg_corner.r*(1.0-xf) + pos_corner.r*xf,
                        i: neg_corner.i*yf + pos_corner.i*(1.0-yf)
                    };
                    let iterations = iterate(c, max_iterations);
                    let (dr,dg,db) = color_palette(iterations);
                    r += dr; g += dg; b += db;
                }
            }
            r /= supersampling*supersampling; g /= supersampling*supersampling; b /= supersampling*supersampling;
            let pos = 3*(y*width + x) as usize;
            pixels[pos+0] = r as u8;
            pixels[pos+1] = g as u8;
            pixels[pos+2] = b as u8;
            if let Result::Ok(true) = cancel_rx.try_recv() {
                return;
            }
        }
    }
}
