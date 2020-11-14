#![forbid(unsafe_code)]

mod canvas;
mod draw_context;
mod draw_primitives;
mod editor;
mod geometry;
mod history;
mod tool;
mod tools;
mod overlay;
mod zoom_overlay;

#[macro_use]
extern crate lazy_static;

use crate::draw_context::DrawContext;
use crate::editor::{Editor, TimeMachineError};
use crate::geometry::Point;
use crate::tool::Tool;
use crate::overlay::{Overlay, EventResponse};
use crate::zoom_overlay::ZoomOverlay;
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::{Keycode, Mod};
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::video::Window;
use sdl2::{EventPump, Sdl};
use sdl2::ttf::Sdl2TtfContext;
use std::cell::RefCell;
use std::error::Error;
use std::fmt;
use std::rc::Rc;
use std::iter::FromIterator;
use std::path::Path;
use std::fs::File;
use std::io::BufWriter;

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

        let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

        Ok(SdlApp {
            sdl_context,
            sdl_canvas,
            event_pump,
            ttf_context,
        })
    }

    pub fn cursor_position(&self) -> Point<i32> {
        let mouse_state = self.event_pump.mouse_state();
        Point::new(mouse_state.x(), mouse_state.y())
    }

    pub fn dimensions(&self) -> (u32, u32) {
        self.sdl_canvas.borrow().window().drawable_size()
    }

    pub fn center(&self) -> Point<i32> {
        let (w, h) = self.dimensions();
        Point::new((w / 2) as i32, (h / 2) as i32)
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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct KeyModifier {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

macro_rules! gen_building_function {
    ($which:ident) => {
        #[allow(dead_code)]
        pub const fn $which(self) -> Self {
            let mut new = self;
            new.$which = true;
            new
        }
    }
}

impl KeyModifier {
    pub const fn new() -> Self {
        Self {
            ctrl: false,
            alt: false,
            shift: false,
        }
    }

    gen_building_function!(ctrl);
    gen_building_function!(shift);
    gen_building_function!(alt);

    pub const fn key(self, key: Keycode) -> KeyWithMod {
        KeyWithMod {
            key,
            modifier: self,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct KeyWithMod {
    pub key: Keycode,
    pub modifier: KeyModifier
}

impl KeyWithMod {
    pub const fn new(key: Keycode, modifier: KeyModifier) -> Self {
        Self { key, modifier }
    }
}

macro_rules! gen_keymod_translation {
    ($src:expr, $sdl_keymods:expr => $result:ident.$keymod:ident) => {
        if $src.intersects(Mod::from_iter($sdl_keymods.iter().copied())) {
            $result = $result.$keymod();
        }
    }
}

impl From<Mod> for KeyModifier {
    fn from(sdl_keymod: Mod) -> Self {
        let mut result = KeyModifier::new();
        gen_keymod_translation!(sdl_keymod, [Mod::LCTRLMOD, Mod::RCTRLMOD] => result.ctrl);
        gen_keymod_translation!(sdl_keymod, [Mod::LSHIFTMOD, Mod::RSHIFTMOD] => result.alt);
        gen_keymod_translation!(sdl_keymod, [Mod::LALTMOD, Mod::RALTMOD] => result.shift);
        result
    }
}

mod hotkey {
    use super::*;

    pub fn handle_undo(oxipaint: &mut OxiPaint) {
        match oxipaint.editor.undo() {
            Ok(_) => {
                println!("Undo OK");
                oxipaint.enqueue_redraw();
            }
            Err(TimeMachineError::AlreadyAtTimeEdge) => {
                println!("Cannot undo at the beginning of the timeline");
            }
            Err(TimeMachineError::TransactionInProgress) => {
                println!("Cannot undo because a drawing action is in progress");
            }
        }
    }

    pub fn handle_redo(oxipaint: &mut OxiPaint) {
        match oxipaint.editor.redo() {
            Ok(_) => {
                println!("Redo OK");
                oxipaint.enqueue_redraw();
            }
            Err(TimeMachineError::AlreadyAtTimeEdge) => {
                println!("Cannot redo at the beginning of the timeline");
            }
            Err(TimeMachineError::TransactionInProgress) => {
                println!("Cannot redo because a drawing action is in progress");
            }
        }
    }

    pub fn save(oxipaint: &mut OxiPaint) -> Result<(), Box<dyn Error>> {
        if let Some(path) = tinyfiledialogs::save_file_dialog("Save file", "image.png") {
            use png::{Encoder, ColorType};
            let file = File::create(Path::new(&path))?;
            let mut file_writer = BufWriter::new(file);
            let canvas = &oxipaint.editor.canvas();
            let mut png_writer = Encoder::new(&mut file_writer, canvas.width(), canvas.height());
            png_writer.set_color(ColorType::RGBA);
            png_writer.write_header()?.write_image_data(&canvas.build_image())?;
            println!("Saved to {}", path);
        } else {
            println!("Saving cancelled");
        }
        Ok(())
    }

    pub fn catch(func: impl Sync + Fn(&mut OxiPaint) -> Result<(), Box<dyn Error>> + 'static) -> HotkeyCallback {
        Box::new(move |oxipaint| {
            match func(oxipaint) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("A non-fatal error occured: {}", e);
                    eprintln!("  -> Detailed information: {:?}", e);
                }
            }
        })
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PressOrRelease {
    Press,
    Release,
}

pub type HotkeyCallback = Box<dyn Fn(&mut OxiPaint) + Sync>;

#[derive(Default)]
pub struct HotkeyAction {
    pub on_press: Option<HotkeyCallback>,
    pub on_release: Option<HotkeyCallback>,
}

impl HotkeyAction {
    pub fn new(
        on_press: Option<HotkeyCallback>,
        on_release: Option<HotkeyCallback>,
    ) -> Self {
        Self {
            on_press: on_press.map(Into::into),
            on_release: on_release.map(Into::into),
        }
    }

    pub fn execute(&self, event: PressOrRelease, oxipaint: &mut OxiPaint) {
        if let Some(func) = match event {
            PressOrRelease::Press => &self.on_press,
            PressOrRelease::Release => &self.on_release,
        } {
            func(oxipaint);
        }
    }
}

lazy_static! {
    pub static ref HOTKEYS: Vec<(KeyWithMod, HotkeyAction)> = {
        vec![
            (
                KeyModifier::new().ctrl().key(Keycode::Z),
                HotkeyAction::new(Some(Box::new(hotkey::handle_undo)), None),
            ),
            (
                KeyModifier::new().ctrl().key(Keycode::Y),
                HotkeyAction::new(Some(Box::new(hotkey::handle_redo)), None),
            ),
            (
                KeyModifier::new().key(Keycode::Space),
                HotkeyAction::new(
                    Some(Box::new(|oxi| oxi.start_scrolling())),
                    Some(Box::new(|oxi| oxi.stop_scrolling())),
                ),
            ),
            (
                KeyModifier::new().ctrl().key(Keycode::S),
                HotkeyAction::new(
                    Some(hotkey::catch(Box::new(hotkey::save))),
                    None
                ),
            ),
        ]
    };
}

fn handle_hotkeys(oxipaint: &mut OxiPaint, key: KeyWithMod, event: PressOrRelease) {
    for (pattern, action) in HOTKEYS.iter() {
        if pattern == &key {
            action.execute(event, oxipaint);
            break;
        }
    }
}

pub struct OxiPaint {
    sdl_app: SdlApp,
    draw_context: DrawContext,
    tools: Vec<Box<dyn Tool>>,
    selected_tool: usize,
    editor: Editor,
    state: OxiPaintState,
    overlay: Option<Box<dyn Overlay>>,
}

impl OxiPaint {
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
            overlay: None,
        })
    }

    fn handle_event(&mut self, event: Event) {
        if let Some(mut overlay) = self.overlay.take() {
            match overlay.handle_event(&event) {
                EventResponse::Close => (),
                EventResponse::Retain => self.overlay = Some(overlay),
            }
        }

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
                keycode: Some(key),
                keymod: sdl_keymod,
                ..
            } => {
                let key_with_mod = KeyWithMod::new(key, sdl_keymod.into());
                handle_hotkeys(self, key_with_mod, PressOrRelease::Press);
            }
            Event::KeyUp {
                keycode: Some(key),
                keymod: sdl_keymod,
                ..
            } => {
                let key_with_mod = KeyWithMod::new(key, sdl_keymod.into());
                handle_hotkeys(self, key_with_mod, PressOrRelease::Release);
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
                    self.set_overlay(ZoomOverlay { zoom: new_scale });
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
                    self.set_overlay(ZoomOverlay { zoom: new_scale });
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
                if let Some(overlay) = &mut self.overlay {
                    // TODO: maybe use proper error handling?
                    overlay.draw(&mut self.sdl_app).unwrap();
                }
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

    fn set_overlay(&mut self, overlay: impl Overlay + 'static) {
        self.overlay = Some(Box::new(overlay));
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
