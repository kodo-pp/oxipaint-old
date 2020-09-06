use crate::canvas::Canvas;
use crate::geometry::Scale;
use crate::history::{DiffDirection, History};
use crate::SdlCanvas;
use sdl2::rect::Rect;
use sdl2::render::TextureCreator;
use sdl2::video::WindowContext;

pub struct Editor {
    canvas: Canvas,
    shadow_data: Vec<u8>,
    history: History,
    in_transaction: bool,
    scale: Scale,
}

impl Editor {
    pub fn new(width: u32, height: u32) -> Editor {
        let canvas = Canvas::new(width, height);
        let shadow_data = canvas.create_shadow_data();
        let history = History::new();
        let in_transaction = false;
        let scale = Scale::Times(1);

        Editor {
            canvas,
            shadow_data,
            history,
            in_transaction,
            scale,
        }
    }

    pub fn scale_up(&mut self) -> Option<Scale> {
        let new_scale = match self.scale {
            Scale::Times(n) => Some(Scale::Times(n + 1)),
        };

        if let Some(s) = new_scale {
            self.scale = s;
        }
        new_scale
    }

    pub fn scale_down(&mut self) -> Option<Scale> {
        let new_scale = match self.scale {
            Scale::Times(0) => unreachable!(),
            Scale::Times(1) => None,
            Scale::Times(n) => Some(Scale::Times(n - 1)),
        };

        if let Some(s) = new_scale {
            self.scale = s;
        }
        new_scale
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

    pub fn draw(
        &self,
        sdl_canvas: &mut SdlCanvas,
        texture_creator: &mut TextureCreator<WindowContext>,
    ) {
        let (w, h) = sdl_canvas.window().drawable_size();
        let viewport = Rect::new(0, 0, w, h);
        self.canvas
            .draw(sdl_canvas, texture_creator, self.scale, viewport);
    }
}
