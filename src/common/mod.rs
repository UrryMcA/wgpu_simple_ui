pub mod vertex;
pub mod primitives;
pub mod types;
pub mod render_box;
pub mod key;
pub mod event;

// Реэкспортируем всё необходимое для обратной совместимости
pub use vertex::{Vertex, DrawCommand};
pub use primitives::Primitives;
