//! Menu bar icon rendering.
//!
//! Icons are drawn at runtime rather than shipped as a sprite sheet: the moon
//! disc is continuous data, not one of a fixed set of pictures, and drawing it
//! means the menu bar can never show a phase that rounds to the wrong picture.

pub mod glyphs;
pub mod moon;
pub mod path;
pub mod render;

pub use render::{chart_icon, graha_icon, moon_icon, Icon, RenderError, Tint};
