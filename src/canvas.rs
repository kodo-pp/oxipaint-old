use crate::geometry::{Point, Scale};
use crate::history::{Diff, DiffDirection, SparsePixelDelta};
use crate::SdlCanvas;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::{Texture, TextureCreator};
use sdl2::video::WindowContext;
use std::convert::TryInto;

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

    pub fn area(&self) -> usize {
        self.width as usize * self.height as usize
    }

    #[allow(dead_code)]
    pub fn get_at(&self, x: u32, y: u32) -> Color {
        self.try_get_at(x, y).unwrap()
    }

    pub fn try_get_at(&self, x: u32, y: u32) -> Option<Color> {
        // TODO: avoid multiple bound checking
        let offset = self.calc_offset(x, y)?;
        let slice = &self.data[offset..offset + Self::BPP];
        Some(Self::color_from_slice(slice))
    }

    fn color_from_slice(slice: &[u8]) -> Color {
        assert!(slice.len() == 4);
        let b = slice[0];
        let g = slice[1];
        let r = slice[2];
        let a = slice[3];
        Color::RGBA(r, g, b, a)
    }

    fn color_to_slice(color: Color, slice: &mut [u8]) {
        assert!(slice.len() == 4);
        slice[0] = color.b;
        slice[1] = color.g;
        slice[2] = color.r;
        slice[3] = color.a;
    }

    pub fn set_at(&mut self, x: u32, y: u32, color: Color) {
        self.try_set_at(x, y, color).unwrap();
    }

    pub fn try_set_at(&mut self, x: u32, y: u32, color: Color) -> Option<()> {
        // TODO: avoid multiple bound checking
        let offset = self.calc_offset(x, y)?;
        let slice = &mut self.data[offset..offset + Self::BPP];
        Self::color_to_slice(color, slice);
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
        scale: Scale,
        viewport: Rect,
    ) {
        let texture = self.sdl_texture(texture_creator, scale, viewport);
        let query = texture.query();
        sdl_canvas
            .copy(
                &texture,
                None,
                Some(Rect::new(0, 0, query.width, query.height)),
            )
            .expect("Failed to draw texture");
    }

    pub fn create_shadow_data(&self) -> Vec<u8> {
        self.data.clone()
    }

    pub fn update_shadow_data(&self, shadow_data: &mut Vec<u8>) {
        shadow_data.clear();
        shadow_data.extend_from_slice(&self.data);
    }

    pub fn compare_shadow_data(&self, shadow_data: &Vec<u8>) -> Diff {
        let mut deltas = Vec::new();
        for index in 0..self.area() {
            let left = index * Self::BPP;
            let right = left + Self::BPP;
            let before = Self::color_from_slice(&shadow_data[left..right]);
            let after = Self::color_from_slice(&self.data[left..right]);
            if before != after {
                deltas.push(SparsePixelDelta {
                    index,
                    before,
                    after,
                });
            }
        }
        Diff::Sparse(deltas)
    }

    pub fn apply_diff(&mut self, diff: &Diff, direction: DiffDirection) {
        match diff {
            Diff::Sparse(deltas) => {
                for delta in deltas.iter() {
                    let left = delta.index * Self::BPP;
                    let right = left + Self::BPP;
                    let slice = &mut self.data[left..right];
                    let color = match direction {
                        DiffDirection::Normal => delta.after,
                        DiffDirection::Reverse => delta.before,
                    };

                    Self::color_to_slice(color, slice);
                }
            }
        }
    }

    pub fn contains_point(&self, point: Point) -> bool {
        point.x >= 0.0
            && point.y >= 0.0
            && point.x < self.width() as f64
            && point.y < self.height() as f64
    }

    fn try_into_x(&self, value: u32) -> Option<u32> {
        Self::try_into_coord(value, self.width)
    }

    fn try_into_y(&self, value: u32) -> Option<u32> {
        Self::try_into_coord(value, self.height)
    }

    fn try_into_coord(value: u32, limit: u32) -> Option<u32> {
        if value < limit {
            Some(value)
        } else {
            None
        }
    }

    fn sdl_texture<'a>(
        &self,
        texture_creator: &'a mut TextureCreator<WindowContext>,
        scale: Scale,
        viewport: Rect,
    ) -> Texture<'a> {
        // TODO: implement a more efficient way of updating the texture (w/o overwriting it
        // completely every time)
        let width = viewport.width().min(scale.apply(self.width));
        let height = viewport.height().min(scale.apply(self.width));
        println!("vp = {:?}", viewport);

        let mut texture = texture_creator
            .create_texture_streaming(PixelFormatEnum::ARGB8888, width, height)
            .expect("Failed to create a texture from the canvas");

        texture
            .with_lock(None, |data, pitch| {
                let pitch_pixels = pitch / Self::BPP;

                for texture_y in 0..height {
                    for texture_x in 0..width {
                        let _ = (|| {
                            macro_rules! method {
                                ($obj:ident.$method:ident) => {
                                    |x| $obj.$method(x)
                                };
                            }

                            let canvas_x = {
                                let unchecked: Option<u32> =
                                    (texture_x as i32 - viewport.left()).try_into().ok();
                                unchecked
                                    .map(method!(scale.unapply))
                                    .and_then(method!(self.try_into_x))?
                            };
                            let canvas_y = {
                                let unchecked: Option<u32> =
                                    (texture_y as i32 - viewport.top()).try_into().ok();
                                unchecked
                                    .map(method!(scale.unapply))
                                    .and_then(method!(self.try_into_y))?
                            };
                            let texture_index =
                                texture_y as usize * pitch_pixels + texture_x as usize;
                            let canvas_index =
                                canvas_y as usize * self.width() as usize + canvas_x as usize;

                            let texture_left = texture_index * Self::BPP;
                            let texture_right = texture_left + Self::BPP;
                            let canvas_left = canvas_index * Self::BPP;
                            let canvas_right = canvas_left + Self::BPP;

                            data[texture_left..texture_right]
                                .copy_from_slice(&self.data[canvas_left..canvas_right]);

                            Some(())
                        })();
                    }
                }
            })
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
