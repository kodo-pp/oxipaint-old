extern crate iced;
use iced::*;


#[derive(Debug, Clone, Copy)]
enum Message {}

type OxiCommand = Command<Message>;
type OxiElement<'a> = Element<'a, Message>;


struct OxiPaint {
    tool_bar: ToolBar,
}

struct OxiPaintFlags {
    tools: Box<dyn Iterator<Item = String>>,
}

impl Application for OxiPaint {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = OxiPaintFlags;

    fn new(flags: Self::Flags) -> (OxiPaint, OxiCommand) {
        let tool_bar = ToolBar::new(flags.tools.map(ToolItem::new).collect(), 0, 200);
        let app = OxiPaint { tool_bar };
        (app, Command::none())
    }

    fn title(&self) -> String {
        "Oxipaint".to_owned()
    }

    fn update(&mut self, _message: Message) -> OxiCommand {
        Command::none()
    }

    fn view(&mut self) -> OxiElement {
        let raw_tool_bar = self.tool_bar.view();
        let contained_tool_bar = Container::new(raw_tool_bar)
            .width(Length::Shrink)
            .center_y()
            .align_x(Align::Start);

        let row = Row::new()
            .spacing(20)
            .padding(10)
            .width(Length::Fill)
            .push(contained_tool_bar);

        Container::new(row)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}


struct ToolBar {
    selected_tool: usize,
    tools: Vec<ToolItem>,
    width: u16,
}

impl ToolBar {
    pub fn new(tools: Vec<ToolItem>, default_tool_index: usize, width: u16) -> ToolBar {
        assert!(default_tool_index < tools.len());
        ToolBar { tools, selected_tool: default_tool_index, width }
    }

    pub fn view(&mut self) -> OxiElement {
        let mut column = Column::new().spacing(10);
        for (i, tool_item) in self.tools.iter_mut().enumerate() {
            let is_selected = i == self.selected_tool;
            let element = tool_item.view(is_selected, self.width);
            column = column.push(element);
        }
        column.into()
    }
}


struct ToolItem {
    label: String,
    button_state: button::State,
}

impl ToolItem {
    pub fn new(label: String) -> ToolItem {
        ToolItem { label, button_state: button::State::new() }
    }

    fn view(&mut self, is_selected: bool, width: u16) -> OxiElement {
        let button_text = Text::new(&self.label);
        let style = SimpleButtonStylesheet::new(
            button::Style {
                background: if is_selected {
                                Some(Color::from_rgb(0.8, 0.8, 1.0).into())
                            } else {
                                None
                            },
                ..button::Style::default()
            }
        );
        let button = Button::new(&mut self.button_state, button_text)
            .style(style)
            .width(Length::Units(width));
        button.into()
    }
}

#[derive(Debug)]
struct SimpleButtonStylesheet {
    style: button::Style,
}

impl SimpleButtonStylesheet {
    pub fn new(style: button::Style) -> SimpleButtonStylesheet {
        SimpleButtonStylesheet { style }
    }
}

impl button::StyleSheet for SimpleButtonStylesheet {
    fn active(&self) -> button::Style {
        button::Style { ..self.style }
    }
}
    

fn main() {
    const NAMES: [&'static str; 4] = [
        "Brush",
        "Pencil",
        "Dragon Blood",
        "Fire",
    ];
    let flags = OxiPaintFlags {
        tools: Box::new(NAMES.iter().copied().map(String::from))
    };
    OxiPaint::run(Settings::with_flags(flags));
}
