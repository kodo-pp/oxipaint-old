use crate::geometry::{Point, Rectangle};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HardLine {
    a: Point,
    b: Point,
    thickness: f64,
}

impl HardLine {
    pub fn new(a: Point, b: Point, thickness: f64) -> HardLine {
        HardLine { a, b, thickness }
    }

    fn points(&self) -> (Point, Point, Point, Point) {
        let normal_x = self.b.y - self.a.y;
        let normal_y = self.a.x - self.b.x;
        let normal_scale = normal_x.hypot(normal_y);
        assert!(normal_scale.abs() > 1e-9);
        let normal_x = normal_x / normal_scale;
        let normal_y = normal_y / normal_scale;

        let p1 = Point::new(
            self.a.x + (normal_x) * self.thickness / 2.0,
            self.a.y + (normal_y) * self.thickness / 2.0,
        );

        let p2 = Point::new(
            self.a.x - normal_x * self.thickness / 2.0,
            self.a.y - normal_y * self.thickness / 2.0,
        );

        let p3 = Point::new(
            self.b.x - normal_x * self.thickness / 2.0,
            self.b.y - normal_y * self.thickness / 2.0,
        );

        let p4 = Point::new(
            self.b.x + normal_x * self.thickness / 2.0,
            self.b.y + normal_y * self.thickness / 2.0,
        );

        (p1, p2, p3, p4)
    }

    fn contains(&self, point: Point) -> bool {
        let (p1, p2, p3, p4) = self.points();
        let v1x = p2.x - p1.x;
        let v1y = p2.y - p1.y;
        let v2x = p3.x - p2.x;
        let v2y = p3.y - p2.y;
        let v3x = p4.x - p3.x;
        let v3y = p4.y - p3.y;
        let v4x = p1.x - p4.x;
        let v4y = p1.y - p4.y;
        let u1x = point.x - p1.x;
        let u1y = point.y - p1.y;
        let u2x = point.x - p2.x;
        let u2y = point.y - p2.y;
        let u3x = point.x - p3.x;
        let u3y = point.y - p3.y;
        let u4x = point.x - p4.x;
        let u4y = point.y - p4.y;
        let cross1 = v1x * u1y - v1y * u1x;
        let cross2 = v2x * u2y - v2y * u2x;
        let cross3 = v3x * u3y - v3y * u3x;
        let cross4 = v4x * u4y - v4y * u4x;
        let sign1 = cross1.signum();
        let sign2 = cross2.signum();
        let sign3 = cross3.signum();
        let sign4 = cross4.signum();
        sign1 == sign2 && sign2 == sign3 && sign3 == sign4
    }

    pub fn draw(&self, put_pixel: &mut impl FnMut(u32, u32)) {
        let rect = self.bounding_box().bounding_int_rectangle();
        for pixel_y in rect.top()..=rect.bottom() {
            if pixel_y < 0 {
                continue;
            }

            for pixel_x in rect.left()..=rect.right() {
                if pixel_x < 0 {
                    continue;
                }

                let point = Point::new(pixel_x as f64 + 0.5, pixel_y as f64 + 0.5);
                if self.contains(point) {
                    put_pixel(pixel_x as u32, pixel_y as u32);
                }
            }
        }
    }

    fn bounding_box(&self) -> Rectangle {
        let (p1, p2, p3, p4) = self.points();
        let min_x = p1.x.min(p2.x).min(p3.x).min(p4.x) - 1.0;
        let min_y = p1.y.min(p2.y).min(p3.y).min(p4.y) - 1.0;
        let max_x = p1.x.max(p2.x).max(p3.x).max(p4.x) + 1.0;
        let max_y = p1.y.max(p2.y).max(p3.y).max(p4.y) + 1.0;

        let left = min_x;
        let top = min_y;
        let width = max_x - min_x;
        let height = max_y - min_y;
        Rectangle::new(left, top, width, height)
    }
}
