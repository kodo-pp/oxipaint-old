use crate::canvas::Canvas;
use crate::geometry::{Point, Scale};
use crate::history::{DiffDirection, History};
use crate::SdlCanvas;
use sdl2::rect::Rect;

use std::cell::RefCell;
use std::rc::Rc;

pub struct Editor {
    canvas: Canvas,
    shadow_data: Vec<u8>,
    history: History,
    in_transaction: bool,
    scale: Scale,
    center: Point,
}

impl Editor {
    pub fn new(width: u32, height: u32, sdl_canvas: Rc<RefCell<SdlCanvas>>) -> Editor {
        let canvas = Canvas::new(width, height, sdl_canvas);
        let shadow_data = canvas.create_shadow_data();
        let history = History::new();
        let in_transaction = false;
        let scale = Scale::Times(1);
        let center = Point::new(width as f64, height as f64).map(|x| x / 2.0);

        Editor {
            canvas,
            shadow_data,
            history,
            in_transaction,
            scale,
            center,
        }
    }

    fn recalc_scale_up(orig_scale: Scale) -> Option<Scale> {
        match orig_scale {
            Scale::Times(n) => Some(Scale::Times(n + 1)),
        }
    }

    fn recalc_scale_down(orig_scale: Scale) -> Option<Scale> {
        match orig_scale {
            Scale::Times(0) => unreachable!(),
            Scale::Times(1) => None,
            Scale::Times(n) => Some(Scale::Times(n - 1)),
        }
    }

    pub fn scale_up(&mut self, stationary_point: Point) -> Option<Scale> {
        let new_scale = Self::recalc_scale_up(self.scale);
        if let Some(s) = new_scale {
            self.rescale(s, stationary_point);
        }
        new_scale
    }

    pub fn scale_down(&mut self, stationary_point: Point) -> Option<Scale> {
        let new_scale = Self::recalc_scale_down(self.scale);
        if let Some(s) = new_scale {
            self.rescale(s, stationary_point);
        }
        new_scale
    }

    fn rescale(&mut self, new_scale: Scale, stationary_point: Point) {
        let orig_scale = self.scale;
        let orig_center = self.center;

        let orig_stationary_point_offset_x = stationary_point.x as f64 - orig_center.x;
        let orig_stationary_point_offset_y = stationary_point.y as f64 - orig_center.y;
        let new_stationary_point_offset_x =
            orig_scale.apply(new_scale.unapply(orig_stationary_point_offset_x));
        let new_stationary_point_offset_y =
            orig_scale.apply(new_scale.unapply(orig_stationary_point_offset_y));

        let new_center = Point::new(
            stationary_point.x - new_stationary_point_offset_x,
            stationary_point.y - new_stationary_point_offset_y,
        )
        .zipmap(
            (self.canvas.width() as f64, self.canvas.height() as f64),
            |coord, lim| coord.max(0.0).min(lim),
        );
        self.center = new_center;
        self.scale = new_scale;
    }

    pub fn canvas(&self) -> &Canvas {
        &self.canvas
    }

    pub fn canvas_mut(&mut self) -> &mut Canvas {
        &mut self.canvas
    }

    pub fn undo(&mut self) -> Option<()> {
        if self.in_transaction {
            panic!("Undo in the middle of transaction is not supported");
        }
        let diff = self.history.undo()?;
        self.canvas.apply_diff(diff, DiffDirection::Reverse);
        Some(())
    }

    pub fn redo(&mut self) -> Option<()> {
        if self.in_transaction {
            panic!("Redo in the middle of transaction is not supported");
        }
        assert!(!self.in_transaction);
        let diff = self.history.redo()?;
        self.canvas.apply_diff(diff, DiffDirection::Normal);
        Some(())
    }

    pub fn begin(&mut self) {
        self.canvas.update_shadow_data(&mut self.shadow_data);
        self.in_transaction = true;
    }

    pub fn end(&mut self) {
        let diff = self.canvas.compare_shadow_data(&self.shadow_data);
        self.history.record(diff);
        self.in_transaction = false;
    }

    pub fn draw(&mut self) {
        let (w, h) = self.canvas.sdl_canvas().borrow().window().drawable_size();
        let (x, y) = self
            .translate_to_image_point(Point::new(0.0, 0.0), w, h)
            .map(|x| x.round() as i32)
            .into();
        let visible_rect = Rect::new(
            x - 1,
            y - 1,
            self.scale.unapply(w) + 2,
            self.scale.unapply(h) + 2,
        );
        self.canvas.draw(
            self.scale,
            visible_rect,
            self.get_left_top_offset_i32(w, h).into(),
        );
    }

    pub fn get_left_top_offset_i32(&self, screen_width: u32, screen_height: u32) -> (i32, i32) {
        let (x, y) = self.get_left_top_offset(screen_width, screen_height);
        (x.round() as i32, y.round() as i32)
    }

    pub fn get_left_top_offset(&self, screen_width: u32, screen_height: u32) -> (f64, f64) {
        let x = screen_width as f64 / 2.0 - self.scale.apply(self.center.x);
        let y = screen_height as f64 / 2.0 - self.scale.apply(self.center.y);
        (x, y)
    }

    pub fn translate_to_image_point(
        &self,
        point: Point,
        screen_width: u32,
        screen_height: u32,
    ) -> Point {
        let (offset_x, offset_y) = self.get_left_top_offset_i32(screen_width, screen_height);
        Point::new(point.x - offset_x as f64, point.y - offset_y as f64)
            .map(|x| self.scale.unapply(x))
    }
}
