#![feature(scoped)]

extern crate gtk;
extern crate gdk;

use std::cell::Cell;
use std::thread;
use gtk::traits::*;
use gtk::signal::Inhibit;

mod complex;
mod mandelbrot;

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
    
    let image = gtk::widgets::Image::new_from_pixbuf(&pixbuf).unwrap();
    window.add(&image);

    window.show_all();

    let scope = thread::scoped(move || {
        mandelbrot::draw(Complex { r: -2.25, i: -0.9140625 }, Complex { r: 1.0, i: 0.9140625 }, 1_000, pixels, width, height);
    });

    // manual main loop so we can refresh the image per iteration
    loop {
        gtk::main_iteration_do(false); // false: don’t block the loop if no events are there
        image.set_from_pixbuf(&pixbuf); // image.queue_draw() doesn’t work for some reason
        if close.get() {
            break;
        }
        thread::sleep_ms(10);
    }

    scope.join();
}
