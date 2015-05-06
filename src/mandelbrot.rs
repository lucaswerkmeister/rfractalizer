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

pub fn draw(pixbuf: gdk::widgets::Pixbuf, max_iterations: i64, neg_corner: Complex, pos_corner: Complex) {
    let w = pixbuf.get_width();
    let h = pixbuf.get_height();
    let rowstride = pixbuf.get_rowstride();
    let n_channels = pixbuf.get_n_channels();
    let mut length = 0u32;
    let mut pixels_ = pixbuf.get_pixels_with_length(&mut length).unwrap();
    let pixels = pixels_.as_mut();
    for x in 0..w {
        for y in 0..h {
            let xf = x as f64 / w as f64;
            let yf = y as f64 / h as f64;
            let c = Complex {
                r: neg_corner.r*(1.0-xf) + pos_corner.r*xf,
                i: neg_corner.i*(1.0-yf) + pos_corner.i*yf
            };
            let iterations = iterate(c, max_iterations);
            let (r,g,b) = if iterations < 0 { (0,0,0) } else { (255,255,255) };
            let pos = (y*rowstride + x*n_channels) as usize;
            pixels[pos+0] = r;
            pixels[pos+1] = g;
            pixels[pos+2] = b;
        }
    }
}
