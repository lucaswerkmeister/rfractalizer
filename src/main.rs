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
    let mut quits = Vec::with_capacity(n_threads as usize);
    let mut corners = Vec::with_capacity(n_threads as usize);
    for i in 0..n_threads {
        let my_slice = slices.pop().unwrap();
        let (cancel_tx,cancel_rx) = channel::<bool>();
        cancels.push(cancel_tx);
        let (quit_tx,quit_rx) = channel::<bool>();
        quits.push(quit_tx);
        let (corner_tx,corner_rx) = channel::<(Complex,Complex)>();
        corners.push(corner_tx);
        threads.push(thread::scoped(move || {
            loop {
                if let Result::Ok(true) = cancel_rx.try_recv() {
                    // ignore
                }
                if let Result::Ok(true) = quit_rx.try_recv() {
                    break;
                }
                if let Result::Ok((neg_corner,pos_corner)) = corner_rx.try_recv() {
                    let my_neg_corner = Complex { r: neg_corner.r, i: neg_corner.i + (pos_corner.i-neg_corner.i)*(i as f64/n_threads as f64) };
                    let my_pos_corner = Complex { r: pos_corner.r, i: neg_corner.i + (pos_corner.i-neg_corner.i)*((i+1) as f64/n_threads as f64) };
                    mandelbrot::draw(my_neg_corner, my_pos_corner, 1_000, my_slice, width, height/n_threads, 2, palettes::color_wheel, &cancel_rx);
                }
                thread::sleep_ms(10);
            }
        }));
    }

    let (point1_tx,point1_rx) = channel::<(f64,f64)>();
    let (point2_tx,point2_rx) = channel::<(f64,f64)>();
    image_events.connect_button_press_event(move |_, button| {
        point1_tx.send((button.x,button.y)).unwrap();
        Inhibit(true)
    });
    image_events.connect_button_release_event(move |_, button| {
        point2_tx.send((button.x,button.y)).unwrap();
        Inhibit(true)
    });
    let mut point1 = (0.0,0.0);
    let mut point2 = (0.0,0.0);

    for corner_tx in &corners {
        corner_tx.send((neg_corner,pos_corner)).unwrap();
    }

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
            let (min_x,min_y,max_x,max_y) = keep_aspect_ratio(width, height, point1, point2);
            let new_neg_corner = Complex {
                r: neg_corner.r * (1.0 - (min_x / width as f64)) + pos_corner.r * (min_x / width as f64),
                i: neg_corner.i * (max_y / height as f64) + pos_corner.i * (1.0 - (max_y / height as f64))
            };
            let new_pos_corner = Complex {
                r: neg_corner.r * (1.0 - (max_x / width as f64)) + pos_corner.r * (max_x / width as f64),
                i: neg_corner.i * (min_y / height as f64) + pos_corner.i * (1.0 - (min_y / height as f64))
            };
            neg_corner = new_neg_corner;
            pos_corner = new_pos_corner;
            for cancel_tx in &cancels {
                cancel_tx.send(true).unwrap();
            }
            for corner_tx in &corners {
                corner_tx.send((neg_corner,pos_corner)).unwrap();
            }
        }
        thread::sleep_ms(10);
    }
    for cancel_tx in cancels {
        cancel_tx.send(true).unwrap();
    }
    for quit_tx in quits {
        quit_tx.send(true).unwrap();
    }
}

/// Given a rectangular area and the coordinates of two corners of a smaller rectangular area within it,
/// returns two corners of a rectangular area with the same upper left corner and the same width or height
/// as the other rectangular area, and the same aspect ratio as the full area.
fn keep_aspect_ratio(width: i32, height: i32, (x1,y1): (f64,f64), (x2,y2): (f64,f64)) -> (f64,f64,f64,f64) {
    let min_x = x1.min(x2);
    let max_x = x1.max(x2);
    let min_y = y1.min(y2);
    let max_y = y1.max(y2);
    let dx = max_x - min_x;
    let dy = max_y - min_y;
    let x_ratio = dx / width as f64;
    let y_ratio = dy / height as f64;
    if x_ratio > y_ratio {
        (min_x, min_y, min_x + y_ratio * width as f64, min_y + dy)
    } else {
        (min_x, min_y, min_x + dx, min_y + x_ratio * height as f64)
    }
}
