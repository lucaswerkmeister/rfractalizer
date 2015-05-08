extern crate gdk;

use complex::Complex;

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
pub fn draw(neg_corner: Complex, pos_corner: Complex, max_iterations: i64, pixels: &mut [u8], width: i32, height: i32) {
    for y in 0..height {
        for x in 0..width {
            let xf = x as f64 / width as f64;
            let yf = y as f64 / height as f64;
            let c = Complex {
                r: neg_corner.r*(1.0-xf) + pos_corner.r*xf,
                i: neg_corner.i*yf + pos_corner.i*(1.0-yf)
            };
            let iterations = iterate(c, max_iterations);
            let (r,g,b) = if iterations < 0 { (0,0,0) } else { (255,255,255) };
            let pos = 3*(y*width + x) as usize;
            pixels[pos+0] = r;
            pixels[pos+1] = g;
            pixels[pos+2] = b;
        }
    }
}
