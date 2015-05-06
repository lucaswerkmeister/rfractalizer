extern crate gtk;
extern crate gdk;

use gtk::traits::*;
use gtk::signal::Inhibit;

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

    let pixbuf = gdk::widgets::Pixbuf::new(gdk::ColorSpace::RGB, /*has_alpha*/false, /*bits_per_sample*/8, width, height).unwrap();
    let image = gtk::widgets::Image::new_from_pixbuf(&pixbuf).unwrap();
    window.add(&image);
    
    window.show_all();
    gtk::main();
}
