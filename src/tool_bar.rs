use crate::tool::{Tool, Tools};
use crate::workarounds::SimpleButtonStylesheet;
use crate::{Message, OxiElement};
use iced::button;
use iced::{Button, Color, Column, Length, Text};

pub struct ToolBar {
    tools: Tools,
    tool_items: Vec<ToolItem>,
    selected_tool_index: Option<usize>,
    width: u16,
}

impl ToolBar {
    pub fn new(tools: Tools, width: u16) -> ToolBar {
        ToolBar {
            tool_items: tools
                .iter()
                .enumerate()
                .map(|(i, tool)| ToolItem::new(tool, i))
                .collect(),
            tools,
            width,
            selected_tool_index: None,
        }
    }

    pub fn view(&mut self) -> OxiElement {
        let mut tool_column = Column::new().spacing(5);
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

    pub fn select_tool(&mut self, tool_index: usize) {
        self.selected_tool_index = Some(tool_index);
    }

    pub fn get_selected_tool_mut(&mut self) -> Option<&mut (dyn Tool + 'static)> {
        /*
        self.selected_tool_index
            .and_then(|index| self.tools.as_vec_mut().get_mut(index))
            .map(|x| x.as_mut())
        */
        if let Some(index) = self.selected_tool_index {
            self.tools.as_vec_mut().get_mut(index).map(|x| x.as_mut())
        } else {
            None
        }
    }
}

struct ToolItem {
    tool_index: usize,
    label: String,
    button_state: button::State,
}

impl ToolItem {
    pub fn new(tool: &dyn Tool, tool_index: usize) -> ToolItem {
        ToolItem {
            label: tool.name(),
            tool_index,
            button_state: button::State::new(),
        }
    }

    fn view(&mut self, is_selected: bool, width: u16) -> OxiElement {
        let button_text = Text::new(self.label.clone());
        let style = SimpleButtonStylesheet::new(button::Style {
            background: if is_selected {
                Some(Color::from_rgb(0.8, 0.8, 1.0).into())
            } else {
                Some(Color::from_rgb(0.8, 0.8, 0.8).into())
            },
            ..button::Style::default()
        });
        let button = Button::new(&mut self.button_state, button_text)
            .style(style)
            .width(Length::Units(width))
            .on_press(Message::SelectTool(self.tool_index));
        button.into()
    }
}
