use crate::TranslatedPoint;
use sdl2::pixels::Color;

#[derive(Debug, Clone, Copy)]
pub struct DrawContext {
    pub primary_color: Color,
    pub cursor_position: TranslatedPoint,
}

impl Default for DrawContext {
    fn default() -> DrawContext {
        DrawContext {
            primary_color: Color::BLACK,
            cursor_position: TranslatedPoint::OutsideWindow,
        }
    }
}
