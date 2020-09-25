#![forbid(unsafe_code)]

mod canvas;
mod draw_context;
mod draw_primitives;
mod editor;
mod geometry;
mod history;
mod tool;
mod tools;

use crate::draw_context::DrawContext;
use crate::editor::{Editor, TimeMachineError};
use crate::geometry::Point;
use crate::tool::Tool;
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::{Keycode, Mod};
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::video::Window;
use sdl2::EventPump;
use sdl2::Sdl;
use std::cell::RefCell;
use std::error::Error;
use std::fmt;
use std::rc::Rc;

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
    pub sdl_context: Sdl,
    pub sdl_canvas: Rc<RefCell<SdlCanvas>>,
    pub event_pump: EventPump,
    pub ttf_context: Sdl2TtfContext,
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

        let sdl_canvas = Rc::new(RefCell::new(
            window
                .into_canvas()
                .accelerated()
                .build()
                .map_err(|e| e.to_string())?,
        ));

        let event_pump = sdl_context.event_pump()?;

        Ok(SdlApp {
            sdl_context,
            sdl_canvas,
            event_pump,
        })
    }

    pub fn cursor_position(&self) -> Point<i32> {
        let mouse_state = self.event_pump.mouse_state();
        Point::new(mouse_state.x(), mouse_state.y())
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct OxiPaintState {
    termination: bool,
    redraw: bool,
    is_scrolling: bool,
}

impl Default for OxiPaintState {
    fn default() -> OxiPaintState {
        OxiPaintState {
            termination: false,
            redraw: true,
            is_scrolling: false,
        }
    }
}

const fn any_keymod_of_two(a: Mod, b: Mod) -> Mod {
    Mod::from_bits_truncate(a.bits() | b.bits())
}

macro_rules! any_keymod_of {
    [$mod:expr] => ($mod);
    [$head:expr, $($tail:expr),+] => (
        any_keymod_of_two($head, any_keymod_of![$($tail),+])
    );
    [$($mods:expr),+,] => (any_keymod_of![$($mods),+]);
}

pub struct OxiPaint {
    sdl_app: SdlApp,
    draw_context: DrawContext,
    tools: Vec<Box<dyn Tool>>,
    selected_tool: usize,
    editor: Editor,
    state: OxiPaintState,
}

impl OxiPaint {
    const HOTKEYS_KEYMOD_MASK: Mod = any_keymod_of![
        Mod::LCTRLMOD,
        Mod::RCTRLMOD,
        Mod::LALTMOD,
        Mod::RALTMOD,
        Mod::LSHIFTMOD,
        Mod::RSHIFTMOD,
    ];

    pub fn new() -> Result<OxiPaint, SdlError> {
        let sdl_app = SdlApp::new()?;
        let draw_context = DrawContext::default();
        let tools = tools::list();
        assert!(!tools.is_empty());
        let selected_tool = 0;
        let editor = Editor::new(800, 600, Rc::clone(&sdl_app.sdl_canvas));
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
            Event::MouseMotion {
                x, y, xrel, yrel, ..
            } => {
                self.update_cursor_position(Some(Point::new(x as u32, y as u32)));
                self.handle_cursor_movement(Some((xrel as f64, yrel as f64)));
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
                keymod,
                ..
            } if [Mod::LCTRLMOD, Mod::RCTRLMOD].contains(&(keymod & Self::HOTKEYS_KEYMOD_MASK)) => {
                match self.editor.undo() {
                    Ok(_) => {
                        println!("Undo OK");
                        self.enqueue_redraw();
                    }
                    Err(TimeMachineError::AlreadyAtTimeEdge) => {
                        println!("Cannot undo at the beginning of the timeline");
                    }
                    Err(TimeMachineError::TransactionInProgress) => {
                        println!("Cannot undo because a drawing action is in progress");
                    }
                }
            }
            Event::KeyDown {
                keycode: Some(Keycode::Y),
                keymod,
                ..
            } if [Mod::LCTRLMOD, Mod::RCTRLMOD].contains(&(keymod & Self::HOTKEYS_KEYMOD_MASK)) => {
                match self.editor.redo() {
                    Ok(_) => {
                        println!("Redo OK");
                        self.enqueue_redraw();
                    }
                    Err(TimeMachineError::AlreadyAtTimeEdge) => {
                        println!("Cannot redo at the beginning of the timeline");
                    }
                    Err(TimeMachineError::TransactionInProgress) => {
                        println!("Cannot redo because a drawing action is in progress");
                    }
                }
            }
            Event::Window { win_event, .. } => match win_event {
                WindowEvent::Leave => {
                    self.update_cursor_position(None);
                    self.handle_cursor_movement(None);
                }
                _ => (),
            },
            Event::MouseWheel { y, .. } if y > 0 => {
                let stationary_point = self
                    .translate_cursor_position(Some(self.sdl_app.cursor_position()))
                    .point()
                    .unwrap();

                if let Some(new_scale) = self.editor.scale_up(stationary_point) {
                    println!("Scale increased to {}", new_scale);
                    self.enqueue_redraw();
                } else {
                    println!("Failed to scale up");
                }
            }
            Event::MouseWheel { y, .. } if y < 0 => {
                let stationary_point = self
                    .translate_cursor_position(Some(self.sdl_app.cursor_position()))
                    .point()
                    .unwrap();

                if let Some(new_scale) = self.editor.scale_down(stationary_point) {
                    println!("Scale decreased to {}", new_scale);
                    self.enqueue_redraw();
                } else {
                    println!("Failed to scale down");
                }
            }
            Event::KeyDown {
                keycode: Some(Keycode::Space),
                ..
            } => {
                self.start_scrolling();
            }
            Event::KeyUp {
                keycode: Some(Keycode::Space),
                ..
            } => {
                self.stop_scrolling();
            }
            _ => (),
        }

        if should_redraw_on(&event) {
            self.enqueue_redraw();
        }
    }

    fn handle_mouse_button_press(&mut self, button: MouseButton) {
        if self.can_draw() {
            let tool = self.tools[self.selected_tool].as_mut();
            if let Redraw::Do =
                tool.on_mouse_button_press(button, &self.draw_context, &mut self.editor)
            {
                self.enqueue_redraw();
            }
        }
    }

    fn handle_mouse_button_release(&mut self, button: MouseButton) {
        if self.can_draw() {
            let tool = self.tools[self.selected_tool].as_mut();
            if let Redraw::Do =
                tool.on_mouse_button_release(button, &self.draw_context, &mut self.editor)
            {
                self.enqueue_redraw();
            }
        }
    }

    fn handle_cursor_movement(&mut self, absolute_delta: Option<(f64, f64)>) {
        if self.is_scrolling() {
            if let Some((adx, ady)) = absolute_delta {
                let k = self.scroll_acceleration();
                let rdx = self.editor.scale().unapply(adx) * k;
                let rdy = self.editor.scale().unapply(ady) * k;
                self.editor.scroll(rdx, rdy);
                self.enqueue_redraw();
            }
        } else if self.can_draw() {
            let tool = self.tools[self.selected_tool].as_mut();
            if let Redraw::Do = tool.on_cursor_move(&self.draw_context, &mut self.editor) {
                self.enqueue_redraw();
            }
        }
    }

    fn scroll_acceleration(&self) -> f64 {
        // TODO: maybe put this value into a config file
        2.0
    }

    fn start_scrolling(&mut self) {
        let mouse_util = self.sdl_app.sdl_context.mouse();
        mouse_util.show_cursor(false);
        mouse_util.set_relative_mouse_mode(true);
        self.state.is_scrolling = true;
        self.enqueue_redraw();
    }

    fn stop_scrolling(&mut self) {
        let mouse_util = self.sdl_app.sdl_context.mouse();
        mouse_util.show_cursor(true);
        mouse_util.set_relative_mouse_mode(false);
        self.state.is_scrolling = false;
        self.enqueue_redraw();
    }

    fn is_scrolling(&self) -> bool {
        self.state.is_scrolling
    }

    fn update_cursor_position(&mut self, position: Option<Point<u32>>) {
        self.draw_context.cursor_position = self.translate_cursor_position(position);
    }

    fn translate_cursor_position(
        &self,
        position: Option<Point<impl Into<f64>>>,
    ) -> TranslatedPoint {
        match position {
            Some(position) => {
                let position: Point<f64> = position.map(|x| x.into());
                let (screen_width, screen_height) = self.get_screen_size();
                let translated_point = self.editor.translate_to_image_point(
                    Point::new(position.x + 0.5, position.y + 0.5),
                    screen_width,
                    screen_height,
                );
                if (0.0..self.editor.canvas().width() as f64).contains(&translated_point.x)
                    && (0.0..self.editor.canvas().height() as f64).contains(&translated_point.y)
                {
                    TranslatedPoint::WithinCanvas(translated_point)
                } else {
                    TranslatedPoint::OutsideCanvas(translated_point)
                }
            }
            None => TranslatedPoint::OutsideWindow,
        }
    }

    fn get_screen_size(&self) -> (u32, u32) {
        self.sdl_app.sdl_canvas.borrow().window().drawable_size()
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
                self.sdl_app
                    .sdl_canvas
                    .borrow_mut()
                    .set_draw_color(Color::BLACK);
                self.sdl_app.sdl_canvas.borrow_mut().clear();
                self.editor.draw();
                self.sdl_app.sdl_canvas.borrow_mut().present();
                self.redrawn();
            }

            let event = self.sdl_app.event_pump.wait_event();
            self.handle_event(event);
        }
    }

    fn can_draw(&self) -> bool {
        !self.is_scrolling()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TranslatedPoint {
    WithinCanvas(Point),
    OutsideCanvas(Point),
    OutsideWindow,
}

impl TranslatedPoint {
    pub fn point(self) -> Option<Point> {
        use TranslatedPoint::*;
        match self {
            WithinCanvas(point) | OutsideCanvas(point) => Some(point),
            OutsideWindow => None,
        }
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
