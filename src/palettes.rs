pub fn black_and_white(iterations: i64) -> (i32,i32,i32) {
    if iterations < 0 {
        (0,0,0)
    } else {
        (255,255,255)
    }
}

pub fn color_wheel(mut iterations: i64) -> (i32,i32,i32) {
    if iterations < 0 {
        return (0,0,0);
    }
    iterations = 10*iterations + 250;
    let sixtile = (iterations / 60) % 6;
    let offset = (((iterations % 60) as f64) * (255 as f64 / 60 as f64)) as i32;
    match sixtile {
        0 => { (255,0,offset) }
        1 => { (255-offset,0,255) }
        2 => { (0,offset,255) }
        3 => { (0,255,255-offset) }
        4 => { (offset,255,0) }
        5 => { (255,255-offset,0) }
        _ => { (0,0,0) } // this should never happen
    }
}
