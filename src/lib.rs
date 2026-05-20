//! 2D UI библиотека на wgpu.

pub mod common;
pub mod primitives_impl;
pub mod texture_manager;
pub mod loader;
pub mod gpu_bitmap_font;
pub mod renderer;
pub mod ui;

// Реэкспорты

pub use primitives_impl::DefaultPrimitives;
pub use texture_manager::TextureManager;
pub use loader::{TextureLoader, FontLoader};
pub use gpu_bitmap_font::{RawGlyph, GpuBitmapFont};
pub use renderer::UiRenderer;
pub use ui::{
    UiManager, Widget, RenderBox, SingleChildRenderObjectWidget,
    MultiChildRenderObjectWidget, LeafRenderObjectWidget,
    Size, Point, Constraints, EdgeInsets, Rect,
    Button, Label, Image, Container, Row, Column, Stack, Positioned,
  
};