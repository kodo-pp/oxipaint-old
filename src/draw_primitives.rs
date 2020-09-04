use crate::geometry::{Point, Rectangle};
use std::mem;

mod hard_line {
    pub enum Intersection {
        Contained,
        Overlaps,
        Disjoint,
    }
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HardLine {
    a: Point,
    b: Point,
    thickness: f64,
}


fn sort2<T: PartialOrd>(mut a: T, mut b: T) -> (T, T) {
    if a > b {
        mem::swap(&mut a, &mut b);
    }
    (a, b)
}

impl HardLine {
    pub fn new(a: Point, b: Point, thickness: f64) -> HardLine {
        HardLine { a, b, thickness }
    }

    fn scanline_points(&self) -> (Point, Point, Point, Point) {
        let normal_x = self.b.y - self.a.y;
        let normal_y = self.a.x - self.b.x;
        let scale = normal_x.hypot(normal_y);
        assert!(scale.abs() > 1e-9);
        let normal_x = normal_x / scale;
        let normal_y = normal_y / scale;

        let p1 = Point::new(
            self.a.x + normal_x * self.thickness / 2.0,
            self.a.y + normal_y * self.thickness / 2.0,
        );

        let p2 = Point::new(
            self.a.x - normal_x * self.thickness / 2.0,
            self.a.y - normal_y * self.thickness / 2.0,
        );

        let p3 = Point::new(
            self.b.x + normal_x * self.thickness / 2.0,
            self.b.y + normal_y * self.thickness / 2.0,
        );

        let p4 = Point::new(
            self.b.x - normal_x * self.thickness / 2.0,
            self.b.y - normal_y * self.thickness / 2.0,
        );
        let mut points = [p1, p2, p3, p4];
        points.sort_by(|p, q| p.y.partial_cmp(&q.y).unwrap());
        (points[0], points[1], points[2], points[3])
    }

    pub fn draw(&self, put_pixel: &mut impl FnMut(u32, u32)) {
        println!("{:?}", self);
        let (top, topmid, bottommid, bottom) = self.scanline_points();
        println!("T {:?}, Tm {:?}, Bm {:?}, B {:?}", top, topmid, bottommid, bottom);
        let mut y = top.y.floor() as i64;
        while y as f64 + 1e-9 < topmid.y {
            if (topmid.y - top.y).abs() < 1e-9 {
                break;
            }
            println!("Loop 1: y = {}", y);
            let dy = y as f64 - top.y + 0.5;
            let k_topmid = dy / (topmid.y - top.y);
            let k_bottommid = dy / (bottommid.y - top.y);
            let dx_topmid = k_topmid * (topmid.x - top.x);
            let dx_bottommid = k_bottommid * (bottommid.x - top.x);
            let x_topmid = top.x + dx_topmid;
            let x_bottommid = top.x + dx_bottommid;
            let (x_left, x_right) = sort2(x_topmid, x_bottommid);
            let x_left = (x_left - 1e-9).round() as u32;
            let x_right = (x_right + 1e-9).round() as u32;
            for x in x_left..=x_right {
                put_pixel(x, y as u32);
            }
            y += 1;
        }
        while y as f64 + 0.5 + 1e-9 < bottommid.y {
            println!("Loop 2: y = {}", y);
            let dy = y as f64 - top.y + 0.5;
            let k_bottommid = dy / (bottommid.y - top.y);
            let dx_bottommid = k_bottommid * (bottommid.x - top.x);
            let x_bottommid = top.x + dx_bottommid;
            let distance_x = (top.y - topmid.y).powi(2) / (top.x - topmid.x) + top.x - topmid.x;
            println!("    dist_x = {}", distance_x);
            let x_topmid = x_bottommid - distance_x;
            let (x_left, x_right) = sort2(x_topmid, x_bottommid);
            println!("xL, xR = {}, {}", x_left, x_right);
            let x_left = (x_left - 1e-9).round() as u32;
            let x_right = (x_right + 1e-9).round() as u32;
            for x in x_left..x_right {
                put_pixel(x, y as u32);
            }
            y += 1;
        }
        while y as f64 + 1e-9 < bottom.y {
            println!("Loop 3: y = {}", y);
            let dy = bottom.y - y as f64 - 0.5;
            let k_topmid = dy / (bottom.y - topmid.y);
            let k_bottommid = dy / (bottom.y - bottommid.y);
            let dx_topmid = k_topmid * (topmid.x - bottom.x);
            let dx_bottommid = k_bottommid * (bottommid.x - bottom.x);
            let x_topmid = bottom.x + dx_topmid;
            let x_bottommid = bottom.x + dx_bottommid;
            let (x_left, x_right) = sort2(x_topmid, x_bottommid);
            let x_left = (x_left - 1e-9).round() as u32;
            let x_right = (x_right + 1e-9).round() as u32;
            for x in x_left..=x_right {
                put_pixel(x, y as u32);
            }
            y += 1;
        }
    }
}
