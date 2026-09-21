use crate::geometry::Rect;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color(u32);

impl Color {
    pub const BLACK: Self = Self::rgb(0, 0, 0);
    pub const WHITE: Self = Self::rgb(255, 255, 255);

    pub const fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self(((red as u32) << 16) | ((green as u32) << 8) | blue as u32)
    }

    pub const fn red(self) -> u8 {
        (self.0 >> 16) as u8
    }

    pub const fn green(self) -> u8 {
        (self.0 >> 8) as u8
    }

    pub const fn blue(self) -> u8 {
        self.0 as u8
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextStyle {
    Regular,
    Bold,
}

pub trait Renderer {
    fn width(&self) -> i32;
    fn height(&self) -> i32;

    fn clear(&mut self, color: Color);
    fn fill_rect(&mut self, rect: Rect, color: Color);
    fn stroke_rect(&mut self, rect: Rect, color: Color, thickness: i32);
    fn text(&mut self, x: i32, y: i32, text: &str, color: Color, style: TextStyle);
}
