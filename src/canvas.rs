use crate::OxiElement;
use iced::image::{Handle, Image};

pub struct Canvas {
    data: Vec<u8>,
    width: u32,
    height: u32,
}

impl Canvas {
    pub fn new(width: u32, height: u32) -> Canvas {
        let data_size = width as usize * height as usize * Self::BPP;
        let mut data = Vec::new();
        data.resize(data_size, 255);
        Canvas {
            width,
            height,
            data,
        }
    }

    pub fn view(&self) -> OxiElement {
        Image::new(Handle::from_pixels(
            self.width,
            self.height,
            self.data.clone(),
        ))
        .into()
    }

    const BPP: usize = 4;
}
