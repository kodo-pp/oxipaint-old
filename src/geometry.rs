use std::ops::{Add, Div, Mul};
use sdl2::rect;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Point<T = f64> {
    pub x: T,
    pub y: T,
}

impl<T> Point<T> {
    pub fn new(x: T, y: T) -> Point<T> {
        Point { x, y }
    }

    pub fn map<O>(self, func: impl Fn(T) -> O) -> Point<O> {
        Point::new(func(self.x), func(self.y))
    }

    pub fn zipmap<A, O>(self, attached: (A, A), func: impl Fn(T, A) -> O) -> Point<O> {
        Point::new(func(self.x, attached.0), func(self.y, attached.1))
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

impl From<rect::Point> for Point<i32> {
    fn from(rect_point: rect::Point) -> Self {
        let tuple: (i32, i32) = rect_point.into();
        tuple.into()
    }
}

impl Into<rect::Point> for Point<i32> {
    fn into(self) -> rect::Point {
        let tuple: (i32, i32) = self.into();
        tuple.into()
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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Scale {
    Times(u32),
}

impl Scale {
    pub fn apply<T: Mul<Output = T> + From<u32>>(self, num: T) -> T {
        match self {
            Scale::Times(n) => num * n.into(),
        }
    }

    pub fn unapply<T: Div<Output = T> + From<u32>>(self, num: T) -> T {
        match self {
            Scale::Times(n) => num / n.into(),
        }
    }

    pub fn to_percentage_string(&self) -> String {
        match self {
            Scale::Times(n) => format!("{}00%", n),
        }
    }
}

impl std::fmt::Display for Scale {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Scale::Times(0) => unreachable!(),
            Scale::Times(n) => write!(fmt, "{}x", n),
        }
    }
}
