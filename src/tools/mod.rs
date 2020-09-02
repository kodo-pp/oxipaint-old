use crate::tool::Tool;

pub mod pencil;

pub fn list() -> Vec<Box<dyn Tool>> {
    vec![Box::new(pencil::Pencil::new())]
}
