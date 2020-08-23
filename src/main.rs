extern crate iced;
use crate::canvas::Canvas;
use crate::tool::Tools;
use crate::tool_bar::ToolBar;
use iced::{container, executor};
use iced::{Align, Application, Color, Command, Container, Element, Length, Row, Settings};

mod canvas;
mod tool;
mod tool_bar;
mod workarounds;

#[derive(Debug, Clone, Copy)]
pub enum Message {
    SelectTool(usize),
}

type OxiCommand = Command<Message>;
type OxiElement<'a> = Element<'a, Message>;

struct OxiPaint {
    tool_bar: ToolBar,
    canvas: Canvas,
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
        let app = OxiPaint { tool_bar, canvas };
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
        let contained_canvas = Container::new(raw_canvas)
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
