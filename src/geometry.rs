use std::ops::Add;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Point<T = f64> {
    pub x: T,
    pub y: T,
}

impl<T> Point<T> {
    pub fn new(x: T, y: T) -> Point<T> {
        Point { x, y }
    }
}

impl<T> From<(T, T)> for Point<T> {
    fn from(tuple: (T, T)) -> Point<T> {
        let (x, y) = tuple;
        Point::new(x, y)
    }
}

impl<T> Into<(T, T)> for Point<T> {
    fn into(self) -> (T, T) {
        (self.x, self.y)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Rectangle<T = f64> {
    left: T,
    top: T,
    width: T,
    height: T,
}

impl<T> Rectangle<T> {
    pub fn new(left: T, top: T, width: T, height: T) -> Rectangle<T> {
        Rectangle {
            left,
            top,
            width,
            height,
        }
    }
}

impl<T: Clone> Rectangle<T> {
    pub fn left(&self) -> T {
        self.left.clone()
    }

    pub fn top(&self) -> T {
        self.top.clone()
    }

    pub fn width(&self) -> T {
        self.width.clone()
    }

    pub fn height(&self) -> T {
        self.height.clone()
    }
}

impl<T: Clone + Add<Output = T>> Rectangle<T> {
    pub fn right(&self) -> T {
        self.left() + self.width()
    }

    pub fn bottom(&self) -> T {
        self.top() + self.height()
    }
}

impl Rectangle<f64> {
    pub fn bounding_int_rectangle(self) -> Rectangle<i64> {
        let left = self.left().floor() as i64;
        let top = self.top().floor() as i64;
        let right = self.right().ceil() as i64;
        let bottom = self.bottom().ceil() as i64;
        let width = right - left;
        let height = bottom - top;
        Rectangle::new(left, top, width, height)
    }
}
