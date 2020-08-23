use iced::button;

#[derive(Debug)]
pub struct SimpleButtonStylesheet {
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
