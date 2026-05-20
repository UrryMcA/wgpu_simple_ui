use std::collections::HashMap;

use crate::common::bitmap_font::{BitmapFont, GlyphInfo};

/// Сырой глиф (из .fnt файла). Не зависит от serde.
#[derive(Debug, Clone)]
pub struct RawGlyph {
    pub id: u32,
    pub width: u16,
    pub height: u16,
    pub xoffset: i16,
    pub yoffset: i16,
    pub xadvance: i16,
    pub x: u16,
    pub y: u16,
}

pub struct GpuBitmapFont {
    texture_id: u64,
    line_height: f32,
    chars: HashMap<u32, GlyphInfo>,
}

impl GpuBitmapFont {
    pub fn new(texture_id: u64, line_height: f32, chars: HashMap<u32, GlyphInfo>) -> Self {
        Self { texture_id, line_height, chars }
    }
}

impl BitmapFont for GpuBitmapFont {
    fn texture_id(&self) -> u64 { self.texture_id }
    fn line_height(&self) -> f32 { self.line_height }
    fn get_glyph(&self, ch: char) -> Option<GlyphInfo> {
        self.chars.get(&(ch as u32)).copied()
    }
}