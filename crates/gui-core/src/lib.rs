#![no_std]

extern crate alloc;

pub mod desktop;
pub mod geometry;
pub mod input;
pub mod render;

pub use desktop::{Desktop, Window, WindowId};
pub use geometry::{Point, Rect, Size};
pub use input::{InputEvent, Key, MouseButton};
pub use render::{Color, Renderer, TextStyle};
