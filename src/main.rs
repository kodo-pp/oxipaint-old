extern crate iced;
extern crate iced_native;
use crate::canvas::Canvas;
use crate::draw_context::DrawContext;
use crate::tool::Tools;
use crate::tool_bar::ToolBar;
use iced::{container, executor, scrollable};
use iced::{Align, Application, Color, Command, Container, Element, Length, Row, Settings};
use iced_native::input::mouse;
use iced_native::Point;

mod canvas;
mod draw_context;
mod tool;
mod tool_bar;
mod tools;
mod workarounds;

#[derive(Debug, Clone, Copy)]
pub enum Message {
    SelectTool(usize),
    CursorMoved(Point),
    MouseButtonPressed {
        cursor_position: Point,
        button: mouse::Button,
    },
    MouseButtonReleased {
        cursor_position: Point,
        button: mouse::Button,
    },
}

type OxiCommand = Command<Message>;
type OxiElement<'a> = Element<'a, Message>;

struct OxiPaint {
    draw_context: DrawContext,
    tool_bar: ToolBar,
    canvas: Canvas,
    canvas_scrollable_state: scrollable::State,
}

impl OxiPaint {
    fn handle_mouse_button_pressed(&mut self, button: mouse::Button) {
        if let Some(tool) = self.tool_bar.get_selected_tool() {
            tool.on_mouse_button_press(button, &self.draw_context);
        }
    }

    fn handle_mouse_button_released(&mut self, button: mouse::Button) {
        if let Some(tool) = self.tool_bar.get_selected_tool() {
            tool.on_mouse_button_release(button, &self.draw_context);
        }
    }

    fn handle_cursor_moved(&mut self) {
        if let Some(tool) = self.tool_bar.get_selected_tool() {
            tool.on_cursor_move(&self.draw_context);
        }
    }
}

#[derive(Default)]
struct OxiPaintFlags {}

impl Application for OxiPaint {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = OxiPaintFlags;

    fn new(_flags: Self::Flags) -> (Self, OxiCommand) {
        let tools = Tools::list_tools();
        let tool_bar = ToolBar::new(tools, 200);
        let canvas = Canvas::new(800, 600);
        let canvas_scrollable_state = scrollable::State::new();
        let app = OxiPaint {
            draw_context: DrawContext::default(),
            tool_bar,
            canvas,
            canvas_scrollable_state,
        };
        (app, OxiCommand::none())
    }

    fn title(&self) -> String {
        "Oxipaint".to_owned()
    }

    fn update(&mut self, message: Message) -> OxiCommand {
        match message {
            Message::SelectTool(tool_index) => {
                self.tool_bar.select_tool(tool_index);
            }
            Message::CursorMoved(point) => {
                self.draw_context.cursor_position = point;
                self.handle_cursor_moved();
            }
            Message::MouseButtonPressed {
                cursor_position,
                button,
            } => {
                self.draw_context.cursor_position = cursor_position;
                self.handle_mouse_button_pressed(button);
            }
            Message::MouseButtonReleased {
                cursor_position,
                button,
            } => {
                self.draw_context.cursor_position = cursor_position;
                self.handle_mouse_button_released(button);
            }
        }
        OxiCommand::none()
    }

    fn view(&mut self) -> OxiElement {
        let raw_tool_bar = self.tool_bar.view();
        let contained_tool_bar = Container::new(raw_tool_bar)
            .width(Length::Shrink)
            .height(Length::Fill)
            .center_y()
            .align_x(Align::Start);

        let raw_canvas = self.canvas.view();
        let scrollable_canvas =
            scrollable::Scrollable::new(&mut self.canvas_scrollable_state).push(raw_canvas);
        let contained_canvas = Container::new(scrollable_canvas)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y();

        let row = Row::new()
            .spacing(20)
            .padding(10)
            .width(Length::Fill)
            .height(Length::Fill)
            .push(contained_tool_bar)
            .push(contained_canvas);

        Container::new(row)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .style(MainWindowStylesheet {})
            .into()
    }
}

struct MainWindowStylesheet {}

impl container::StyleSheet for MainWindowStylesheet {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(Color::from_rgb(0.7, 0.7, 0.7).into()),
            ..container::Style::default()
        }
    }
}

fn main() {
    OxiPaint::run(Settings::default());
}
