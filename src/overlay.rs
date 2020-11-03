use crate::{SdlApp, SdlError};
use sdl2::event::Event;
use sdl2::rect::Rect;
use sdl2::pixels::Color;

pub enum EventResponse {
    Close,
    Retain,
}

pub trait Overlay {
    fn handle_event(&mut self, event: &Event) -> EventResponse;
    fn draw(&mut self, sdl_app: &mut SdlApp) -> Result<(), SdlError>;
}

pub trait SimpleOverlay {
    fn handle_event(&mut self, event: &Event) -> EventResponse;
    fn draw(&mut self, sdl_app: &mut SdlApp, rect: Rect) -> Result<(), SdlError>;
    fn dimensions() -> (u32, u32);
}

impl<T: SimpleOverlay> Overlay for T {
    fn handle_event(&mut self, event: &Event) -> EventResponse {
        self.handle_event(event)
    }

    fn draw(&mut self, sdl_app: &mut SdlApp) -> Result<(), SdlError> {
        let inner_rect = {
            let center = sdl_app.center();
            let mut canvas = sdl_app.sdl_canvas.borrow_mut();
            let (width, height) = <Self as SimpleOverlay>::dimensions();

            let outer_rect = Rect::from_center(center, width + 2, height + 2);
            canvas.set_draw_color(Color::BLACK);
            canvas.draw_rect(outer_rect)?;

            let inner_rect = Rect::from_center(center, width, height);
            canvas.set_draw_color(Color::WHITE);
            canvas.fill_rect(inner_rect)?;

            inner_rect
        };

        self.draw(sdl_app, inner_rect)
    }
}
