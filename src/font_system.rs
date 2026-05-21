// src/font_system.rs
use std::collections::HashMap;
use crate::common::types::{BitmapFont, Size, GlyphInfo, UColor, Rect, TexCoords};
use crate::common::vertex::Vertex;
use crate::common::primitives::Primitives;

/// Централизованное хранилище шрифтов.
pub struct FontSystem {
    fonts: HashMap<String, Box<dyn BitmapFont>>,
    default_font_name: Option<String>,
}

impl FontSystem {
    pub fn new() -> Self {
        Self {
            fonts: HashMap::new(),
            default_font_name: None,
        }
    }

    /// Добавляет шрифт в систему.
    pub fn add_font(&mut self, name: String, font: Box<dyn BitmapFont>) {
        if self.default_font_name.is_none() {
            self.default_font_name = Some(name.clone());
        }
        self.fonts.insert(name, font);
    }

    /// Устанавливает шрифт по умолчанию.
    pub fn set_default_font(&mut self, name: &str) -> bool {
        if self.fonts.contains_key(name) {
            self.default_font_name = Some(name.to_string());
            true
        } else {
            false
        }
    }

    /// Получить шрифт по имени.
    pub fn get_font(&self, name: &str) -> Option<&dyn BitmapFont> {
        self.fonts.get(name).map(|b| b.as_ref())
    }

    /// Получить шрифт по умолчанию.
    pub fn default_font(&self) -> Option<&dyn BitmapFont> {
        self.default_font_name.as_ref().and_then(|name| self.get_font(name))
    }

    // ---------- Измерение текста ----------
    pub fn measure_text(&self, text: &str, font_size: f32, max_width: f32) -> Size {
        let font = match self.default_font() {
            Some(f) => f,
            None => return Size::zero(),
        };
        self.measure_text_with_font(font, text, font_size, max_width)
    }

    pub fn measure_text_with_font(&self, font: &dyn BitmapFont, text: &str, font_size: f32, max_width: f32) -> Size {
        let scale = font_size / font.line_height();
        let mut max_line_width: f32 = 0.0;
        let mut line_width: f32 = 0.0;
        let mut lines = 1;

        for ch in text.chars() {
            if ch == '\n' {
                max_line_width = max_line_width.max(line_width);
                line_width = 0.0;
                lines += 1;
                continue;
            }
            if let Some(glyph) = font.get_glyph(ch) {
                let advance = glyph.xadvance * scale;
                if line_width + advance > max_width && max_width > 0.0 {
                    max_line_width = max_line_width.max(line_width);
                    line_width = advance;
                    lines += 1;
                } else {
                    line_width += advance;
                }
            }
        }
        max_line_width = max_line_width.max(line_width);
        let height = font.line_height() * scale * lines as f32;
        Size::new(max_line_width, height)
    }

    // ---------- Генерация вершин текста ----------
    /// Генерирует вершины для текста, используя шрифт по умолчанию.
    pub fn generate_text_vertices(
        &self,
        text: &str,
        x: f32,
        y: f32,
        scale: f32,
        color: UColor,
        primitives: &dyn Primitives,
    ) -> Vec<Vertex> {
        let font = match self.default_font() {
            Some(f) => f,
            None => return Vec::new(),
        };
        self.generate_text_vertices_with_font(font, text, x, y, scale, color, primitives)
    }

    /// Генерирует вершины для текста с указанным шрифтом.
    pub fn generate_text_vertices_with_font(
        &self,
        font: &dyn BitmapFont,
        text: &str,
        mut x: f32,
        y: f32,
        scale: f32,
        color: UColor,
        primitives: &dyn Primitives,
    ) -> Vec<Vertex> {
        let mut vertices = Vec::new();
        let mut pen_x = x;

        for ch in text.chars() {
            if let Some(glyph) = font.get_glyph(ch) {
                let w = glyph.width * scale;
                let h = glyph.height * scale;
                let rect = Rect::new(
                    pen_x + glyph.xoffset * scale,
                    y + glyph.yoffset * scale,
                    w,
                    h,
                );
                let tex = TexCoords {
                    u0: glyph.u0,
                    v0: glyph.v0,
                    u1: glyph.u1,
                    v1: glyph.v1,
                };
                let verts = primitives.textured_rect_vertices(rect, tex, color);
                vertices.extend(verts);
                pen_x += glyph.xadvance * scale;
            }
        }
        vertices
    }
}

impl Default for FontSystem {
    fn default() -> Self {
        Self::new()
    }
}