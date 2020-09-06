mod canvas;
mod draw_context;
mod draw_primitives;
mod editor;
mod geometry;
mod history;
mod tool;
mod tools;

use crate::draw_context::DrawContext;
use crate::editor::Editor;
use crate::geometry::Point;
use crate::tool::Tool;
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::{Keycode, Mod};
use sdl2::mouse::{MouseButton, MouseWheelDirection};
use sdl2::pixels::Color;
use sdl2::render::TextureCreator;
use sdl2::video::{Window, WindowContext};
use sdl2::EventPump;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SdlError(String);

impl From<String> for SdlError {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl fmt::Display for SdlError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(formatter, "SDL error: {}", self.0)
    }
}

impl Error for SdlError {}

pub type SdlCanvas = sdl2::render::Canvas<Window>;

pub struct SdlApp {
    sdl_canvas: SdlCanvas,
    event_pump: EventPump,
    texture_creator: TextureCreator<WindowContext>,
}

impl SdlApp {
    pub fn new() -> Result<SdlApp, SdlError> {
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;
        let window = video_subsystem
            .window("Window Title", 800, 600)
            .position_centered()
            .resizable()
            .build()
            .map_err(|e| e.to_string())?;

        let sdl_canvas = window
            .into_canvas()
            .accelerated()
            .build()
            .map_err(|e| e.to_string())?;

        let texture_creator = sdl_canvas.texture_creator();

        let event_pump = sdl_context.event_pump()?;

        Ok(SdlApp {
            sdl_canvas,
            event_pump,
            texture_creator,
        })
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct OxiPaintState {
    termination: bool,
    redraw: bool,
}

impl Default for OxiPaintState {
    fn default() -> OxiPaintState {
        OxiPaintState {
            termination: false,
            redraw: true,
        }
    }
}

mod adhoc_oxipaint {
    use super::*;

    pub struct OxiPaint {
        sdl_app: SdlApp,
        draw_context: DrawContext,
        tools: Vec<Box<dyn Tool>>,
        selected_tool: Option<usize>,
        editor: Editor,
        state: OxiPaintState,
    }

    impl OxiPaint {
        pub fn new() -> Result<OxiPaint, SdlError> {
            let sdl_app = SdlApp::new()?;
            let draw_context = DrawContext::default();
            let tools = tools::list();
            assert!(!tools.is_empty());
            let selected_tool = Some(0);
            let editor = Editor::new(800, 600);
            let state = OxiPaintState::default();

            Ok(OxiPaint {
                sdl_app,
                draw_context,
                tools,
                selected_tool,
                editor,
                state,
            })
        }

        fn handle_event(&mut self, event: Event) {
            match event {
                Event::Quit { .. } => {
                    self.enqueue_termination();
                }
                Event::MouseMotion { x, y, .. } => {
                    self.update_cursor_position(Some(Point::new(x as u32, y as u32)));
                    self.handle_cursor_movement();
                }
                Event::MouseButtonDown {
                    x, y, mouse_btn, ..
                } => {
                    self.update_cursor_position(Some(Point::new(x as u32, y as u32)));
                    self.handle_mouse_button_press(mouse_btn);
                }
                Event::MouseButtonUp {
                    x, y, mouse_btn, ..
                } => {
                    self.update_cursor_position(Some(Point::new(x as u32, y as u32)));
                    self.handle_mouse_button_release(mouse_btn);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Z),
                    keymod: Mod::LCTRLMOD,
                    ..
                } => {
                    if self.editor.undo().is_some() {
                        println!("Undo OK");
                        self.enqueue_redraw();
                    } else {
                        println!("Cannot undo at the beginning of the timeline");
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Y),
                    keymod: Mod::LCTRLMOD,
                    ..
                } => {
                    if self.editor.redo().is_some() {
                        println!("Redo OK");
                        self.enqueue_redraw();
                    } else {
                        println!("Cannot redo, since travelling into the future is not supported");
                    }
                }
                Event::Window { win_event, .. } => match win_event {
                    WindowEvent::Leave => {
                        self.update_cursor_position(None);
                        self.handle_cursor_movement();
                    }
                    _ => (),
                }
                Event::MouseWheel { y, .. } if y > 0 => {
                    if let Some(new_scale) = self.editor.scale_up() {
                        println!("Scale increased to {}", new_scale);
                        self.enqueue_redraw();
                    } else {
                        println!("Failed to scale up");
                    }
                }
                Event::MouseWheel { y, .. } if y < 0 => {
                    if let Some(new_scale) = self.editor.scale_down() {
                        println!("Scale decreased to {}", new_scale);
                        self.enqueue_redraw();
                    } else {
                        println!("Failed to scale down");
                    }
                }
                _ => (),
            }

            if should_redraw_on(&event) {
                self.enqueue_redraw();
            }
        }

        fn handle_mouse_button_press(&mut self, button: MouseButton) {
            if let Some(index) = self.selected_tool {
                let tool = self.tools[index].as_mut();
                if let Redraw::Do =
                    tool.on_mouse_button_press(button, &self.draw_context, &mut self.editor)
                {
                    self.enqueue_redraw();
                }
            }
        }

        fn handle_mouse_button_release(&mut self, button: MouseButton) {
            if let Some(index) = self.selected_tool {
                let tool = self.tools[index].as_mut();
                if let Redraw::Do =
                    tool.on_mouse_button_release(button, &self.draw_context, &mut self.editor)
                {
                    self.enqueue_redraw();
                }
            }
        }

        fn handle_cursor_movement(&mut self) {
            if let Some(index) = self.selected_tool {
                let tool = self.tools[index].as_mut();
                if let Redraw::Do = tool.on_cursor_move(&self.draw_context, &mut self.editor) {
                    self.enqueue_redraw();
                }
            }
        }

        fn update_cursor_position(&mut self, position: Option<Point<u32>>) {
            self.draw_context.cursor_position = self.translate_cursor_position(position);
        }

        fn translate_cursor_position(&self, position: Option<Point<u32>>) -> TranslatedPoint {
            match position {
                Some(position) => {
                    let translated_x = position.x as f64 + 0.5;
                    let translated_y = position.y as f64 + 0.5;
                    let translated_point = Point::new(translated_x, translated_y);
                    if position.x < self.editor.canvas().width()
                        && position.y < self.editor.canvas().height()
                    {
                        TranslatedPoint::WithinCanvas(translated_point)
                    } else {
                        TranslatedPoint::OutsideCanvas(translated_point)
                    }
                }
                None => TranslatedPoint::OutsideWindow,
            }
        }

        fn enqueue_termination(&mut self) {
            self.state.termination = true;
        }

        fn enqueue_redraw(&mut self) {
            self.state.redraw = true;
        }

        fn should_terminate(&self) -> bool {
            self.state.termination
        }

        fn should_redraw(&self) -> bool {
            self.state.redraw
        }

        fn redrawn(&mut self) {
            self.state.redraw = false;
        }

        pub fn run(mut self) {
            while !self.should_terminate() {
                if self.should_redraw() {
                    self.sdl_app.sdl_canvas.set_draw_color(Color::BLACK);
                    self.sdl_app.sdl_canvas.clear();
                    self.editor.draw(
                        &mut self.sdl_app.sdl_canvas,
                        &mut self.sdl_app.texture_creator,
                    );
                    self.sdl_app.sdl_canvas.present();
                    self.redrawn();
                }

                let event = self.sdl_app.event_pump.wait_event();
                self.handle_event(event);
            }
        }
    }
}

pub use adhoc_oxipaint::OxiPaint;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TranslatedPoint {
    WithinCanvas(Point),
    OutsideCanvas(Point),
    OutsideWindow,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Redraw {
    Do,
    Dont,
}

fn should_redraw_on(event: &Event) -> bool {
    match event {
        Event::Window { win_event, .. } => match win_event {
            WindowEvent::SizeChanged { .. } => true,
            _ => false,
        },
        _ => false,
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let oxipaint = OxiPaint::new()?;
    oxipaint.run();
    Ok(())
}
