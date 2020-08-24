use crate::draw_context::DrawContext;
use crate::tool::Tool;
use iced_native::input::mouse;

pub struct Pencil {}

impl Pencil {
    pub fn new() -> Pencil {
        Pencil {}
    }
}

impl Tool for Pencil {
    fn name(&self) -> String {
        "Pencil".to_owned()
    }

    fn on_mouse_button_press(&self, button: mouse::Button, context: &DrawContext) {
        println!(
            "Mouse press! button = {:?}, context = {:?}",
            button, context
        );
    }

    fn on_mouse_button_release(&self, button: mouse::Button, context: &DrawContext) {
        println!(
            "Mouse release! button = {:?}, context = {:?}",
            button, context
        );
    }

    fn on_cursor_move(&self, context: &DrawContext) {
        println!("Mouse move! context = {:?}", context);
    }
}
