use crate::canvas::Canvas;
use crate::draw_context::DrawContext;
use crate::draw_primitives;
use crate::tool::Tool;
use crate::{Point, Redraw};
use sdl2::mouse::MouseButton;

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
        button: MouseButton,
        context: &DrawContext,
        _canvas: &mut Canvas,
    ) -> Redraw {
        match button {
            MouseButton::Left => {
                let point = context.cursor_position;
                self.state = PencilState::Active { last_point: point };
            }
            _ => (),
        }
        Redraw::Dont
    }

    fn on_mouse_button_release(
        &mut self,
        button: MouseButton,
        _context: &DrawContext,
        _canvas: &mut Canvas,
    ) -> Redraw {
        match button {
            MouseButton::Left => {
                self.state = PencilState::Inactive;
            }
            _ => (),
        }
        Redraw::Dont
    }

    fn on_cursor_move(&mut self, context: &DrawContext, canvas: &mut Canvas) -> Redraw {
        use PencilState::*;
        let state_copy = self.state;
        match state_copy {
            Inactive => Redraw::Dont,
            Active { last_point: None } => {
                // Previous point outside the canvas
                self.state = Active {
                    last_point: context.cursor_position,
                };
                Redraw::Dont
            }
            Active {
                last_point: Some(last_point),
            } => {
                if let Some(current_point) = context.cursor_position {
                    // Previous and current points within the canvas
                    draw_primitives::hard_line(last_point, current_point, |x, y| {
                        canvas.set_at(x, y, context.primary_color)
                    });
                    canvas.set_at(last_point.x, last_point.y, context.primary_color);
                    canvas.set_at(current_point.x, current_point.y, context.primary_color);
                    self.state = Active {
                        last_point: Some(current_point),
                    };
                    Redraw::Do
                } else {
                    // Previous point within, but current point outside the canvas
                    self.state = Active { last_point: None };
                    Redraw::Dont
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum PencilState {
    Inactive,
    Active { last_point: Option<Point> },
}

impl PencilState {
    fn new() -> PencilState {
        PencilState::Inactive
    }
}
