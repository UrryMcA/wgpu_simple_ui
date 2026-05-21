//! 2D UI библиотека на wgpu.

pub mod common;
pub mod layout;
pub mod primitives_impl;
pub mod texture_manager;
pub mod font_system;
pub mod renderer;
pub mod ui;
pub mod drag_drop_manager;
pub mod ui_manager;

// Реэкспорты

pub use primitives_impl::DefaultPrimitives;
pub use texture_manager::TextureManager;
pub use renderer::UiRenderer;
pub use ui::*;