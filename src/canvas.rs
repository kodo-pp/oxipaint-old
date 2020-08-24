use crate::{Message, OxiElement};
use iced::image::{Handle, Image};
use iced::{Color, Length, Point};
use iced_native::input::{mouse, ButtonState};
use iced_native::layout::{Layout, Limits, Node};
use iced_native::renderer::Renderer as RendererTrait;
use iced_native::{Clipboard, Event, Hasher, Widget};
use iced_wgpu::Renderer;

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
        CanvasWidget::new(Image::new(Handle::from_pixels(
            self.width,
            self.height,
            self.data.clone(),
        )))
        .into()
    }

    pub fn get_at(&self, x: u32, y: u32) -> Color {
        // TODO: avoid multiple bound checking
        let offset = self.calc_offset(x, y).unwrap();
        let slice = &self.data[offset..offset + Self::BPP];
        let b = slice[0];
        let g = slice[1];
        let r = slice[2];
        let a = slice[3];
        Color::from_rgba8(r, g, b, a as f32 / 256.0)
    }

    pub fn set_at(&mut self, x: u32, y: u32, color: Color) {
        // TODO: avoid multiple bound checking
        let offset = self.calc_offset(x, y).unwrap();
        let slice = &mut self.data[offset..offset + Self::BPP];
        let (r, g, b, a) = unpack_color_u8(color);
        slice[0] = b;
        slice[1] = g;
        slice[2] = r;
        slice[3] = a;
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

struct CanvasWidget {
    image_widget: Image,
}

impl CanvasWidget {
    pub fn new(image_widget: Image) -> CanvasWidget {
        CanvasWidget { image_widget }
    }
}

impl<'a> Into<OxiElement<'a>> for CanvasWidget {
    fn into(self) -> OxiElement<'a> {
        OxiElement::new(self)
    }
}

impl Widget<Message, Renderer> for CanvasWidget {
    fn width(&self) -> Length {
        Widget::<Message, Renderer>::width(&self.image_widget)
    }

    fn height(&self) -> Length {
        Widget::<Message, Renderer>::height(&self.image_widget)
    }

    fn layout(&self, renderer: &Renderer, limits: &Limits) -> Node {
        Widget::<Message, Renderer>::layout(&self.image_widget, renderer, limits)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        defaults: &<Renderer as RendererTrait>::Defaults,
        layout: Layout,
        cursor_position: Point,
    ) -> <Renderer as RendererTrait>::Output {
        Widget::<Message, Renderer>::draw(
            &self.image_widget,
            renderer,
            defaults,
            layout,
            cursor_position,
        )
    }

    fn hash_layout(&self, state: &mut Hasher) {
        Widget::<Message, Renderer>::hash_layout(&self.image_widget, state);
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout,
        cursor_position: Point,
        messages: &mut Vec<Message>,
        renderer: &Renderer,
        clipboard: Option<&dyn Clipboard>,
    ) {
        if let Event::Mouse(event) = event {
            use mouse::Event::*;
            match event {
                CursorMoved { .. } => {
                    // Position taken from `cursor_position`
                    if let Some(point) = recalc_coords(cursor_position, layout) {
                        messages.push(Message::CursorMoved(point));
                    }
                }
                Input { state, button } => {
                    if let Some(cursor_position) = recalc_coords(cursor_position, layout) {
                        let message = match state {
                            ButtonState::Pressed => Message::MouseButtonPressed {
                                cursor_position,
                                button,
                            },
                            ButtonState::Released => Message::MouseButtonReleased {
                                cursor_position,
                                button,
                            },
                        };
                        messages.push(message);
                    }
                }
                _ => (),
            }
        }

        self.image_widget.on_event(
            event,
            layout,
            cursor_position,
            messages,
            renderer,
            clipboard,
        );
    }
}

fn recalc_coords(cursor_position: Point, layout: Layout) -> Option<Point> {
    let bounding_rect = layout.bounds();
    if bounding_rect.contains(cursor_position) {
        let x = cursor_position.x - bounding_rect.x;
        let y = cursor_position.y - bounding_rect.y;
        Some(Point::new(x, y))
    } else {
        None
    }
}

fn unpack_color_u8(color: Color) -> (u8, u8, u8, u8) {
    #[inline]
    fn transform(value: f32) -> u8 {
        (value * 256.0).floor() as u8
    }

    (
        transform(color.r),
        transform(color.g),
        transform(color.b),
        transform(color.a),
    )
}
