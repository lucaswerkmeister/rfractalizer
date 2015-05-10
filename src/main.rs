#![feature(scoped)]

extern crate gtk;
extern crate gdk;

use std::thread;
use std::result::Result;
use std::sync::mpsc::channel;
use std::vec::Vec;
use gtk::traits::*;
use gtk::signal::Inhibit;

mod complex;
mod mandelbrot;
mod palettes;

use complex::Complex;

fn main() {
    let width:i32 = 1280;
    let height:i32 = 720;
    
    gtk::init();
    
    let window = gtk::widgets::Window::new(gtk::WindowType::TopLevel).unwrap();
    window.set_title("RFractalizer");
    window.set_window_position(gtk::WindowPosition::Center);

    let (close_tx,close_rx) = channel::<bool>();
    window.connect_delete_event(move |_, _| {
        close_tx.send(true).unwrap();
        Inhibit(true)
    });

    let pixbuf = unsafe { gdk::pixbuf::Pixbuf::new(gdk::ColorSpace::RGB, /*has_alpha*/false, /*bits_per_sample*/8, width, height).unwrap() };
    let mut pixels = unsafe { pixbuf.get_pixels() };

    let n_threads = 8;
    let mut slices = Vec::with_capacity(n_threads as usize);
    for i in 0..n_threads {
        slices.push(unsafe { std::slice::from_raw_parts_mut(&mut pixels[(i*width*height*3/n_threads) as usize] as *mut u8, (width*height*3/n_threads) as usize) });
    }
    
    let image = gtk::widgets::Image::new_from_pixbuf(&pixbuf).unwrap();
    let image_events = gtk::widgets::EventBox::new().unwrap(); // GtkImage receives no events, needs to be wrapped
    image_events.add(&image);
    window.add(&image_events);

    window.show_all();

    let mut neg_corner = Complex { r: -2.25, i: -0.9140625 };
    let mut pos_corner = Complex { r:   1.0, i:  0.9140625 };

    let mut threads = Vec::with_capacity(n_threads as usize);
    let mut cancels = Vec::with_capacity(n_threads as usize);
    let mut corners = Vec::with_capacity(n_threads as usize);
    for i in 0..n_threads {
        let my_slice = slices.pop().unwrap();
        let (cancel_tx,cancel_rx) = channel::<bool>();
        cancels.push(cancel_tx);
        let (corner_tx,corner_rx) = channel::<(Complex,Complex)>();
        corners.push(corner_tx);
        threads.push(thread::scoped(move || {
            loop {
                if let Result::Ok((neg_corner,pos_corner)) = corner_rx.try_recv() {
                    let my_neg_corner = Complex { r: neg_corner.r, i: neg_corner.i + (pos_corner.i-neg_corner.i)*(i as f64/n_threads as f64) };
                    let my_pos_corner = Complex { r: pos_corner.r, i: neg_corner.i + (pos_corner.i-neg_corner.i)*((i+1) as f64/n_threads as f64) };
                    mandelbrot::draw(my_neg_corner, my_pos_corner, 1_000, my_slice, width, height/n_threads, 2, palettes::color_wheel, &cancel_rx);
                }
                if let Result::Ok(true) = cancel_rx.try_recv() {
                    break;
                }
                thread::sleep_ms(10);
            }
        }));
    }

    let (point1_tx,point1_rx) = channel::<Complex>();
    let (point2_tx,point2_rx) = channel::<Complex>();
    image_events.connect_button_press_event(move |_, button| {
        let point1 = Complex {
            r: neg_corner.r * (1.0 - (button.x / width as f64)) + pos_corner.r * (button.x / width as f64),
            i: neg_corner.i * (button.y / height as f64) + pos_corner.i * (1.0 - (button.y / height as f64))
        };
        point1_tx.send(point1).unwrap();
        Inhibit(true)
    });
    image_events.connect_button_release_event(move |_, button| {
        let point2 = Complex {
            r: neg_corner.r * (1.0 - (button.x / width as f64)) + pos_corner.r * (button.x / width as f64),
            i: neg_corner.i * (button.y / height as f64) + pos_corner.i * (1.0 - (button.y / height as f64))
        };
        point2_tx.send(point2).unwrap();
        Inhibit(true)
    });
    let mut point1 = neg_corner;
    let mut point2 = pos_corner;

    // manual main loop so we can refresh the image per iteration
    loop {
        gtk::main_iteration_do(false); // false: don’t block the loop if no events are there
        image.set_from_pixbuf(&pixbuf); // image.queue_draw() doesn’t work for some reason
        if let Result::Ok(true) = close_rx.try_recv() {
            break;
        }
        if let Result::Ok(new_point1) = point1_rx.try_recv() {
            point1 = new_point1;
        }
        if let Result::Ok(new_point2) = point2_rx.try_recv() {
            point2 = new_point2;
            neg_corner = Complex { r: point1.r.min(point2.r), i: point1.i.min(point2.i) };
            pos_corner = Complex { r: point1.r.max(point2.r), i: point1.i.max(point2.i) };
            for corner_tx in corners {
                corner_tx.send((neg_corner,pos_corner)).unwrap();
            }
        }
        thread::sleep_ms(10);
    }
    for cancel_tx in cancels {
        cancel_tx.send(true); // warning: .unwrap() will panic if the receiver was destroyed (calculation done)
    }
}
