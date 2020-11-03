use crate::geometry::Scale;
use crate::overlay::{EventResponse, SimpleOverlay};
use crate::{SdlApp, SdlError};
use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::ttf::{Font, Sdl2TtfContext};
use font_kit::source::SystemSource;
use font_kit::properties::Properties;
use font_kit::family_name::FamilyName;
use font_kit::handle::Handle;

pub struct ZoomOverlay {
    pub zoom: Scale,
}

fn load_font<'ttf>(ttf_context: &'ttf Sdl2TtfContext) -> Result<Font<'ttf, 'static>, String> {
    let font_source = SystemSource::new();
    let handle = font_source
        .select_best_match(&[FamilyName::SansSerif], &Properties::new())
        .map_err(|e| e.to_string())?;
    match handle {
        Handle::Path { path, .. } => ttf_context.load_font(path, 24).map_err(|e| { panic!(e) }),
        _ => panic!("Expected Handle::Path"),
    }
}

impl SimpleOverlay for ZoomOverlay {
    fn draw(&mut self, sdl_app: &mut SdlApp, rect: Rect) -> Result<(), SdlError> {
        // TODO: don't load a font every time
        let font = load_font(&sdl_app.ttf_context)?;
        let surface = font
            .render(&self.zoom.to_percentage_string())
            .solid(Color::BLACK)
            .map_err(|e| e.to_string())?;

        let texture_creator = sdl_app
            .sdl_canvas
            .borrow_mut()
            .texture_creator();

        let texture = texture_creator
            .create_texture_from_surface(surface)
            .map_err(|e| e.to_string())?;

        let texture_rect = {
            let q = texture.query();
            Rect::from_center(rect.center(), q.width, q.height)
        };

        sdl_app
            .sdl_canvas
            .borrow_mut()
            .copy(&texture, None, Some(texture_rect))?;

        Ok(())
    }

    fn handle_event(&mut self, event: &Event) -> EventResponse {
        match event {
            Event::MouseMotion { .. }
            | Event::MouseWheel { .. }
            | Event::MouseButtonDown { .. }
            | Event::KeyDown { .. } => EventResponse::Close,
            _ => EventResponse::Retain,
        }
    }

    fn dimensions() -> (u32, u32) {
        (100, 50)
    }
}
