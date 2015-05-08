#![feature(scoped)]

extern crate gtk;
extern crate gdk;

use std::cell::Cell;
use std::thread;
use std::vec::Vec;
use gtk::traits::*;
use gtk::signal::Inhibit;

mod complex;
mod mandelbrot;
mod palettes;

use complex::Complex;

fn main() {
    let width:i32 = 1920;
    let height:i32 = 1080;
    
    gtk::init();
    
    let window = gtk::widgets::Window::new(gtk::WindowType::TopLevel).unwrap();
    window.set_title("RFractalizer");
    window.set_window_position(gtk::WindowPosition::Center);

    let close = Cell::new(false);
    window.connect_delete_event(|_, _| {
        close.set(true);
        Inhibit(true)
    });

    let pixbuf = gdk::widgets::Pixbuf::new(gdk::ColorSpace::RGB, /*has_alpha*/false, /*bits_per_sample*/8, width, height).unwrap();
    let mut length = 0_u32;
    let mut p = pixbuf.get_pixels_with_length(&mut length).unwrap();
    let mut pixels = p.as_mut();

    let n_threads = 8;
    let mut slices = Vec::with_capacity(n_threads as usize);
    for i in 0..n_threads {
        slices.push(unsafe { std::slice::from_raw_parts_mut(&mut pixels[(i*width*height*3/n_threads) as usize] as *mut u8, (width*height*3/n_threads) as usize) });
    }
    
    let image = gtk::widgets::Image::new_from_pixbuf(&pixbuf).unwrap();
    window.add(&image);

    window.show_all();

    let neg_corner = Complex { r: -2.25, i: -0.9140625 };
    let pos_corner = Complex { r:   1.0, i:  0.9140625 };

    let mut threads = Vec::with_capacity(n_threads as usize);
    for i in 0..n_threads {
        let my_slice = slices.pop().unwrap();
        threads.push(thread::scoped(move || {
            let my_neg_corner = Complex { r: neg_corner.r, i: neg_corner.i + (pos_corner.i-neg_corner.i)*(i as f64/n_threads as f64) };
            let my_pos_corner = Complex { r: pos_corner.r, i: neg_corner.i + (pos_corner.i-neg_corner.i)*((i+1) as f64/n_threads as f64) };
            mandelbrot::draw(my_neg_corner, my_pos_corner, 1_000, my_slice, width, height/n_threads, 2, palettes::black_and_white);
        }));
    }

    // manual main loop so we can refresh the image per iteration
    loop {
        gtk::main_iteration_do(false); // false: don’t block the loop if no events are there
        image.set_from_pixbuf(&pixbuf); // image.queue_draw() doesn’t work for some reason
        if close.get() {
            break;
        }
        thread::sleep_ms(10);
    }
}
