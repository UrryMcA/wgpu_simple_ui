pub mod vertex;
pub mod primitives;
pub mod bitmap_font;

// Реэкспортируем всё необходимое для обратной совместимости
pub use vertex::{Vertex, DrawCommand};
pub use primitives::Primitives;
pub use bitmap_font::{BitmapFont, GlyphInfo, generate_text_vertices};