use crate::Point;
use std::mem;

pub fn hard_line(a: Point, b: Point, mut put_pixel: impl FnMut(u32, u32)) {
    if a.x == b.x {
        vertical_hard_line(a.x, a.y, b.y, put_pixel);
    } else if a.y == b.y {
        horizontal_hard_line(a.y, a.x, b.x, put_pixel);
    } else {
        // TODO: Fix this shit
        let ax = a.x as i64;
        let bx = b.x as i64;
        let ay = a.y as i64;
        let by = b.y as i64;
        let w = (ax - bx).abs() as u32;
        let h = (ay - by).abs() as u32;
        if w > h {
            let k = (by - ay) as f64 / (bx - ax) as f64;
            for t in 0..=w {
                let dx = (bx - ax).signum() as i64 * t as i64;
                let dy = dx as f64 * k;
                let x_rounded = ax + dx;
                let y_rounded = (ay as f64 + dy + 0.5).floor();
                put_pixel(x_rounded as u32, y_rounded as u32);
            }
        } else {
            let k = (bx - ax) as f64 / (by - ay) as f64;
            for t in 0..=h {
                let dy = (by - ay).signum() as i64 * t as i64;
                let dx = dy as f64 * k;
                let y_rounded = ay + dy;
                let x_rounded = (ax as f64 + dx + 0.5).floor();
                put_pixel(x_rounded as u32, y_rounded as u32);
            }
        }
    }
}

fn vertical_hard_line(x: u32, mut y1: u32, mut y2: u32, mut put_pixel: impl FnMut(u32, u32)) {
    if y1 > y2 {
        mem::swap(&mut y1, &mut y2);
    }

    for y in y1..=y2 {
        put_pixel(x, y);
    }
}

fn horizontal_hard_line(y: u32, mut x1: u32, mut x2: u32, mut put_pixel: impl FnMut(u32, u32)) {
    if x1 > x2 {
        mem::swap(&mut x1, &mut x2);
    }

    for x in x1..=x2 {
        put_pixel(x, y);
    }
}
