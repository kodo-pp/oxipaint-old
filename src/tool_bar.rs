use crate::tool::{Tool, Tools};
use crate::workarounds::SimpleButtonStylesheet;
use crate::OxiElement;
use iced::button;
use iced::{Button, Color, Column, Length, Text};

pub struct ToolBar {
    tool_items: Vec<ToolItem>,
    selected_tool_index: Option<usize>,
    width: u16,
}

impl ToolBar {
    pub fn new(tools: &Tools, width: u16) -> ToolBar {
        ToolBar {
            tool_items: tools.iter().map(ToolItem::new).collect(),
            width,
            selected_tool_index: None,
        }
    }

    pub fn view(&mut self) -> OxiElement {
        let mut tool_column = Column::new().spacing(10);
        for (i, tool_item) in self.tool_items.iter_mut().enumerate() {
            let is_selected = {
                if let Some(index) = self.selected_tool_index {
                    i == index
                } else {
                    false
                }
            };

            let element = tool_item.view(is_selected, self.width);
            tool_column = tool_column.push(element);
        }
        tool_column.into()
    }
}

struct ToolItem {
    label: String,
    button_state: button::State,
}

impl ToolItem {
    pub fn new(tool: &(impl Tool + ?Sized)) -> ToolItem {
        ToolItem {
            label: tool.name(),
            button_state: button::State::new(),
        }
    }

    fn view(&mut self, is_selected: bool, width: u16) -> OxiElement {
        let button_text = Text::new(&self.label);
        let style = SimpleButtonStylesheet::new(button::Style {
            background: if is_selected {
                Some(Color::from_rgb(0.8, 0.8, 1.0).into())
            } else {
                None
            },
            ..button::Style::default()
        });
        let button = Button::new(&mut self.button_state, button_text)
            .style(style)
            .width(Length::Units(width));
        button.into()
    }
}
