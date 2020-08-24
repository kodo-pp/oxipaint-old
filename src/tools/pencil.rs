use crate::canvas::Canvas;
use crate::draw_context::DrawContext;
use crate::tool::Tool;
use iced::{Point, Vector};
use iced_native::input::mouse;

pub struct Pencil {
    state: PencilState,
}

impl Pencil {
    pub fn new() -> Pencil {
        Pencil {
            state: PencilState::new(),
        }
    }
}

impl Tool for Pencil {
    fn name(&self) -> String {
        "Pencil".to_owned()
    }

    fn on_mouse_button_press(
        &mut self,
        button: mouse::Button,
        context: &DrawContext,
        _canvas: &mut Canvas,
    ) {
        match button {
            mouse::Button::Left => {
                let point = to_u32_vector(context.cursor_position);
                self.state = PencilState::Active { last_point: point };
            }
            _ => (),
        }
        /*
        println!(
            "Mouse press! button = {:?}, context = {:?}",
            button, context
        );
        */
    }

    fn on_mouse_button_release(
        &mut self,
        button: mouse::Button,
        _context: &DrawContext,
        _canvas: &mut Canvas,
    ) {
        match button {
            mouse::Button::Left => {
                self.state = PencilState::Inactive;
            }
            _ => (),
        }
        /*
        println!(
            "Mouse release! button = {:?}, context = {:?}",
            button, context
        );
        */
    }

    fn on_cursor_move(&mut self, context: &DrawContext, canvas: &mut Canvas) {
        use PencilState::*;
        let state_copy = self.state;
        match state_copy {
            Inactive => (),
            Active { last_point } => {
                canvas.set_at(last_point.x, last_point.y, context.primary_color);
                let point = to_u32_vector(context.cursor_position);
                canvas.set_at(point.x, point.y, context.primary_color);
                self.state = Active { last_point: point };
            }
        }
        /*
        println!("Mouse move! context = {:?}", context);
        */
    }
}

fn to_u32_vector(point: Point) -> Vector<u32> {
    let x = point.x as u32;
    let y = point.y as u32;
    Vector::new(x, y)
}

#[derive(Debug, Clone, Copy)]
enum PencilState {
    Inactive,
    Active { last_point: Vector<u32> },
}

impl PencilState {
    fn new() -> PencilState {
        PencilState::Inactive
    }
}
