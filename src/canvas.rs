use crate::SdlCanvas;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::{Texture, TextureCreator};
use sdl2::video::WindowContext;

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

    #[allow(dead_code)]
    pub fn get_at(&self, x: u32, y: u32) -> Color {
        self.try_get_at(x, y).unwrap()
    }

    pub fn try_get_at(&self, x: u32, y: u32) -> Option<Color> {
        // TODO: avoid multiple bound checking
        let offset = self.calc_offset(x, y)?;
        let slice = &self.data[offset..offset + Self::BPP];
        let b = slice[0];
        let g = slice[1];
        let r = slice[2];
        let a = slice[3];
        Some(Color::RGBA(r, g, b, a))
    }

    pub fn set_at(&mut self, x: u32, y: u32, color: Color) {
        self.try_set_at(x, y, color).unwrap();
    }

    pub fn try_set_at(&mut self, x: u32, y: u32, color: Color) -> Option<()> {
        // TODO: avoid multiple bound checking
        let offset = self.calc_offset(x, y)?;
        let slice = &mut self.data[offset..offset + Self::BPP];
        slice[0] = color.b;
        slice[1] = color.g;
        slice[2] = color.r;
        slice[3] = color.a;
        Some(())
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn draw(
        &self,
        sdl_canvas: &mut SdlCanvas,
        texture_creator: &mut TextureCreator<WindowContext>,
    ) {
        let texture = self.sdl_texture(texture_creator);
        let dest_rect = Rect::new(0, 0, self.width(), self.height());
        sdl_canvas
            .copy(&texture, None, Some(dest_rect))
            .expect("Failed to draw texture");
    }

    fn sdl_texture<'a>(
        &self,
        texture_creator: &'a mut TextureCreator<WindowContext>,
    ) -> Texture<'a> {
        // TODO: implement a more efficient way of updating the texture (w/o overwriting it
        // completely every time)
        let mut texture = texture_creator
            .create_texture_streaming(PixelFormatEnum::ARGB8888, self.width(), self.height())
            .expect("Failed to create a texture from the canvas");

        texture
            .update(None, &self.data, self.width as usize * Self::BPP)
            .expect("Failed to fill the texture with the image data");

        texture
    }

    fn calc_offset(&self, x: u32, y: u32) -> Option<usize> {
        if x >= self.width || y >= self.height {
            None
        } else {
            let x = x as usize;
            let y = y as usize;
            let w = self.width as usize;
            let b = Self::BPP;
            Some((y * w + x) * b)
        }
    }

    const BPP: usize = 4;
}
