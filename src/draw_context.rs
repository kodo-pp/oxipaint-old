use iced::{Color, Point};

#[derive(Debug, Clone, Copy)]
pub struct DrawContext {
    pub primary_color: Color,
    pub cursor_position: Point,
}

impl Default for DrawContext {
    fn default() -> DrawContext {
        DrawContext {
            primary_color: Color::BLACK,
            cursor_position: Point::new(0.0, 0.0),
        }
    }
}
