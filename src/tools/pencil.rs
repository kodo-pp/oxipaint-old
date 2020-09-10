use crate::draw_context::DrawContext;
use crate::draw_primitives::*;
use crate::editor::Editor;
use crate::tool::Tool;
use crate::{Redraw, TranslatedPoint};
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
        editor: &mut Editor,
    ) -> Redraw {
        match button {
            MouseButton::Left => {
                let point = context.cursor_position;
                self.state = PencilState::Active { last_point: point };
                editor.begin();
            }
            _ => (),
        }
        Redraw::Dont
    }

    fn on_mouse_button_release(
        &mut self,
        button: MouseButton,
        _context: &DrawContext,
        editor: &mut Editor,
    ) -> Redraw {
        match button {
            MouseButton::Left => {
                self.state = PencilState::Inactive;
                editor.end();
            }
            _ => (),
        }
        Redraw::Dont
    }

    fn on_cursor_move(&mut self, context: &DrawContext, editor: &mut Editor) -> Redraw {
        use PencilState::*;
        use TranslatedPoint::*;
        let state_copy = self.state;
        match state_copy {
            Inactive => Redraw::Dont,
            Active {
                last_point: OutsideWindow,
            } => {
                // Previous point outside the editor
                self.state = Active {
                    last_point: context.cursor_position,
                };
                Redraw::Dont
            }
            Active {
                last_point: WithinCanvas(last_point),
            }
            | Active {
                last_point: OutsideCanvas(last_point),
            } => {
                match context.cursor_position {
                    WithinCanvas(current_point) | OutsideCanvas(current_point) => {
                        // Previous and current points within the window
                        let contains_last_point = editor.canvas().contains_point(last_point);
                        if contains_last_point {
                            editor.canvas_mut().set_at(
                                last_point.x as u32,
                                last_point.y as u32,
                                context.primary_color,
                            );
                        }
                        if editor.canvas().contains_point(current_point) {
                            editor.canvas_mut().try_set_at(
                                current_point.x as u32,
                                current_point.y as u32,
                                context.primary_color,
                            );
                        }

                        if let Some(line) = HardLine::try_new(last_point, current_point, 1.0) {
                            line.draw(&mut |x, y| {
                                editor.canvas_mut().try_set_at(x, y, context.primary_color);
                            });
                        }
                        self.state = Active {
                            last_point: WithinCanvas(current_point),
                        };
                        Redraw::Do
                    }
                    OutsideWindow => {
                        // Previous point within, but current point outside the window
                        self.state = Active {
                            last_point: OutsideWindow,
                        };
                        Redraw::Dont
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PencilState {
    Inactive,
    Active { last_point: TranslatedPoint },
}

impl PencilState {
    fn new() -> PencilState {
        PencilState::Inactive
    }
}
