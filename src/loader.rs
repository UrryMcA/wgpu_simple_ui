use crate::gpu_bitmap_font::RawGlyph;

/// Загрузчик текстур: возвращает RGBA данные и размеры.
pub trait TextureLoader {
    fn load_texture_rgba(&self, name: &str) -> Option<(Vec<u8>, u32, u32)>;
}

/// Загрузчик шрифтов: возвращает RGBA атласа, размеры атласа и список сырых глифов.
pub trait FontLoader {
    fn load_font_data(&self, name: &str) -> Option<(Vec<u8>, u32, u32, Vec<RawGlyph>)>;
}