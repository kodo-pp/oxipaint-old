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

    pub fn scale(&self) -> Scale {
        self.scale
    }

    pub fn scroll(&mut self, delta_x: f64, delta_y: f64) {
        let width = self.canvas.width() as f64;
        let height = self.canvas.width() as f64;
        self.center = self
            .center
            .zipmap((delta_x, delta_y), |t, dt| t + dt)
            .zipmap((width, height), |t, lim| t.max(0.0).min(lim));
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

    pub fn undo(&mut self) -> Result<(), TimeMachineError> {
        if self.in_transaction {
            return Err(TimeMachineError::TransactionInProgress);
        }
        let diff = self
            .history
            .undo()
            .ok_or(TimeMachineError::AlreadyAtTimeEdge)?;
        self.canvas.apply_diff(diff, DiffDirection::Reverse);
        Ok(())
    }

    pub fn redo(&mut self) -> Result<(), TimeMachineError> {
        if self.in_transaction {
            return Err(TimeMachineError::TransactionInProgress);
        }
        let diff = self
            .history
            .redo()
            .ok_or(TimeMachineError::AlreadyAtTimeEdge)?;
        self.canvas.apply_diff(diff, DiffDirection::Normal);
        Ok(())
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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TimeMachineError {
    TransactionInProgress,
    AlreadyAtTimeEdge,
}
