extern crate gtk;

use gtk::traits::*;
use gtk::widgets;
use gtk::signal::Inhibit;

fn main() {
    gtk::init();
    
    let window = widgets::Window::new(gtk::WindowType::TopLevel).unwrap();
    window.set_title("RFractalizer");
    window.set_window_position(gtk::WindowPosition::Center);

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(true)
    });
    
    window.show_all();
    gtk::main();
}
