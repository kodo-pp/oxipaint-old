use crate::draw_context::DrawContext;
use crate::editor::Editor;
use crate::Redraw;
use sdl2::mouse::MouseButton;

pub trait Tool {
    fn name(&self) -> String;

    fn on_mouse_button_press(
        &mut self,
        _button: MouseButton,
        _context: &DrawContext,
        _editor: &mut Editor,
    ) -> Redraw {
        Redraw::Dont
    }

    fn on_mouse_button_release(
        &mut self,
        _button: MouseButton,
        _context: &DrawContext,
        _editor: &mut Editor,
    ) -> Redraw {
        Redraw::Dont
    }

    fn on_cursor_move(&mut self, _context: &DrawContext, _editor: &mut Editor) -> Redraw {
        Redraw::Dont
    }
}
