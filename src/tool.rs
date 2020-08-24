use crate::draw_context::DrawContext;
use crate::tools::pencil::Pencil;
use iced_native::input::mouse;

pub trait Tool {
    fn name(&self) -> String;

    fn on_mouse_button_press(&self, _button: mouse::Button, _context: &DrawContext) {}
    fn on_mouse_button_release(&self, _button: mouse::Button, _context: &DrawContext) {}
    fn on_cursor_move(&self, _context: &DrawContext) {}
}

pub struct Tools {
    tools: Vec<Box<dyn Tool>>,
}

impl Tools {
    pub fn new() -> Tools {
        Tools { tools: Vec::new() }
    }

    pub fn list_tools() -> Tools {
        let mut tools = Tools::new();
        tools.push(Pencil::new());
        tools.push(DummyTool::from("Dragon blood".to_owned()));
        tools.push(DummyTool::from("Infernal flame".to_owned()));
        tools
    }

    pub fn push(&mut self, tool: impl Tool + 'static) {
        self.tools.push(Box::new(tool));
    }

    pub fn iter(&self) -> impl Iterator<Item = &dyn Tool> {
        self.tools.iter().map(|boxed| boxed.as_ref())
    }

    pub fn as_vec(&self) -> &Vec<Box<dyn Tool>> {
        &self.tools
    }
}

#[derive(Debug, Clone)]
struct DummyTool {
    name: String,
}

impl From<String> for DummyTool {
    fn from(name: String) -> DummyTool {
        DummyTool { name }
    }
}

impl Tool for DummyTool {
    fn name(&self) -> String {
        self.name.clone()
    }
}
