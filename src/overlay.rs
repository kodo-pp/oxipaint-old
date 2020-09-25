use crate::{SdlApp, SdlError};
use sdl2::event::Event;

pub enum EventResponse {
    Close,
    Leave,
}

pub trait Overlay {
    fn draw(&mut self, sdl_app: &SdlApp) -> Result<(), SdlError>;
    fn handle_event(&mut self, event: Event) -> EventResponse;
}
