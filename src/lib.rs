//! 2D UI библиотека на wgpu.

pub mod common;
pub mod layout;
pub mod primitives_impl;
pub mod texture_manager;
pub mod renderer;
pub mod ui;

// Реэкспорты

pub use primitives_impl::DefaultPrimitives;
pub use texture_manager::TextureManager;
pub use renderer::UiRenderer;
pub use ui::*;