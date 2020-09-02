mod canvas;
mod draw_context;
mod draw_primitives;
mod tool;
mod tools;

use crate::canvas::Canvas;
use crate::draw_context::DrawContext;
use crate::tool::Tool;
use sdl2::event::{Event, WindowEvent};
use sdl2::mouse::MouseButton;
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
        canvas: Canvas,
        state: OxiPaintState,
    }

    impl OxiPaint {
        pub fn new() -> Result<OxiPaint, SdlError> {
            let sdl_app = SdlApp::new()?;
            let draw_context = DrawContext::default();
            let tools = tools::list();
            assert!(!tools.is_empty());
            let selected_tool = Some(0);
            let canvas = Canvas::new(800, 600);
            let state = OxiPaintState::default();

            Ok(OxiPaint {
                sdl_app,
                draw_context,
                tools,
                selected_tool,
                canvas,
                state,
            })
        }

        fn handle_event(&mut self, event: Event) {
            match event {
                Event::Quit { .. } => {
                    self.enqueue_termination();
                }
                Event::MouseMotion { x, y, .. } => {
                    self.update_cursor_position(Point::new(x as u32, y as u32));
                    self.handle_cursor_movement();
                }
                Event::MouseButtonDown {
                    x, y, mouse_btn, ..
                } => {
                    self.update_cursor_position(Point::new(x as u32, y as u32));
                    self.handle_mouse_button_press(mouse_btn);
                }
                Event::MouseButtonUp {
                    x, y, mouse_btn, ..
                } => {
                    self.update_cursor_position(Point::new(x as u32, y as u32));
                    self.handle_mouse_button_release(mouse_btn);
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
                    tool.on_mouse_button_press(button, &self.draw_context, &mut self.canvas)
                {
                    self.enqueue_redraw();
                }
            }
        }

        fn handle_mouse_button_release(&mut self, button: MouseButton) {
            if let Some(index) = self.selected_tool {
                let tool = self.tools[index].as_mut();
                if let Redraw::Do =
                    tool.on_mouse_button_release(button, &self.draw_context, &mut self.canvas)
                {
                    self.enqueue_redraw();
                }
            }
        }

        fn handle_cursor_movement(&mut self) {
            if let Some(index) = self.selected_tool {
                let tool = self.tools[index].as_mut();
                if let Redraw::Do = tool.on_cursor_move(&self.draw_context, &mut self.canvas) {
                    self.enqueue_redraw();
                }
            }
        }

        fn update_cursor_position(&mut self, position: Point) {
            self.draw_context.cursor_position = self.translate_cursor_position(position);
        }

        fn translate_cursor_position(&self, position: Point) -> Option<Point> {
            if position.x < self.canvas.width() && position.y < self.canvas.height() {
                Some(position)
            } else {
                None
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
                    self.canvas.draw(
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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Point {
    x: u32,
    y: u32,
}

impl Point {
    pub fn new(x: u32, y: u32) -> Point {
        Point { x, y }
    }
}

impl From<(u32, u32)> for Point {
    fn from(tuple: (u32, u32)) -> Point {
        let (x, y) = tuple;
        Point::new(x, y)
    }
}

impl Into<(u32, u32)> for Point {
    fn into(self) -> (u32, u32) {
        (self.x, self.y)
    }
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
