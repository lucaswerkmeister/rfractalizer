extern crate gtk;
extern crate gdk;

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

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(true)
    });

    let pixbuf = gdk::widgets::Pixbuf::new(gdk::ColorSpace::RGB, /*has_alpha*/true, /*bits_per_sample*/8, width, height).unwrap();
    let image = gtk::widgets::Image::new_from_pixbuf(&pixbuf).unwrap();
    window.add(&image);

    mandelbrot::draw(pixbuf, 1_000, Complex { r: -2.25, i: -0.9140625 }, Complex { r: 1.0, i: 0.9140625 });
    
    window.show_all();
    gtk::main();
}
