use std::collections::HashMap;
use std::cell::RefCell;
use crate::common::types::{BitmapFont, Size, UColor, Rect, TexCoords, CachedGlyph};
use crate::common::vertex::Vertex;
use crate::common::primitives::Primitives;

pub struct FontSystem {
    fonts: HashMap<String, Box<dyn BitmapFont>>,
    default_font_name: Option<String>,
    glyph_cache: RefCell<HashMap<(u64, char, u32), CachedGlyph>>,
}

impl FontSystem {
    pub fn new() -> Self {
        Self {
            fonts: HashMap::new(),
            default_font_name: None,
            glyph_cache: RefCell::new(HashMap::new()),
        }
    }

    pub fn add_font(&mut self, name: String, font: Box<dyn BitmapFont>) {
        if self.default_font_name.is_none() {
            self.default_font_name = Some(name.clone());
        }
        self.fonts.insert(name, font);
    }

    pub fn set_default_font(&mut self, name: &str) -> bool {
        if self.fonts.contains_key(name) {
            self.default_font_name = Some(name.to_string());
            true
        } else {
            false
        }
    }

    pub fn get_font(&self, name: &str) -> Option<&dyn BitmapFont> {
        self.fonts.get(name).map(|b| b.as_ref())
    }

    pub fn default_font(&self) -> Option<&dyn BitmapFont> {
        self.default_font_name.as_ref().and_then(|name| self.get_font(name))
    }

    // ---------- Кэширование глифов ----------
    pub fn get_cached_glyph(
        &self,
        font: &dyn BitmapFont,
        ch: char,
        font_size: f32,
        primitives: &dyn Primitives,
    ) -> Option<CachedGlyph> {
        let texture_id = font.texture_id();
        let size_key = font_size.round() as u32;
        let key = (texture_id, ch, size_key);

        {
            let cache = self.glyph_cache.borrow();
            if let Some(glyph) = cache.get(&key) {
                return Some(glyph.clone());
            }
        }

        let glyph_info = font.get_glyph(ch)?;
        let rect = Rect::new(0.0, 0.0, glyph_info.width, glyph_info.height);
        let tex_coords = TexCoords::new(glyph_info.u0, glyph_info.v0, glyph_info.u1, glyph_info.v1);
        let color = UColor([1.0, 1.0, 1.0, 1.0]);
        let (verts, _) = primitives.textured_rect_vertices_indices(rect, tex_coords, color);
        assert_eq!(verts.len(), 4, "textured_rect_vertices_indices must return exactly 4 vertices");
        let mut cached_vertices = [verts[0]; 4];
        cached_vertices.copy_from_slice(&verts);

        let cached = CachedGlyph {
            vertices: cached_vertices,
            width: glyph_info.width,
            height: glyph_info.height,
            xoffset: glyph_info.xoffset,
            yoffset: glyph_info.yoffset,
            xadvance: glyph_info.xadvance,
        };

        self.glyph_cache.borrow_mut().insert(key, cached.clone());
        Some(cached)
    }

    pub fn clear_glyph_cache(&self) {
        self.glyph_cache.borrow_mut().clear();
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

    // ---------- Генерация вершин и индексов текста ----------
    #[allow(clippy::too_many_arguments)]
    pub fn generate_text_vertices_with_font(
        &self,
        font: &dyn BitmapFont,
        text: &str,
        x: f32,
        y: f32,
        font_size: f32,
        color: UColor,
        primitives: &dyn Primitives,
    ) -> (Vec<Vertex>, Vec<u32>) {
        let scale = font_size / font.line_height();
        let mut all_verts = Vec::new();
        let mut all_indices = Vec::new();
        let mut pen_x = x;

        for ch in text.chars() {
            if ch == '\n' {
                pen_x = x; // упрощённо
                continue;
            }
            let cached = match self.get_cached_glyph(font, ch, font_size, primitives) {
                Some(g) => g,
                None => continue,
            };
            let base_idx = all_verts.len() as u32;
            for orig_vert in cached.vertices.iter() {
                let mut vert = *orig_vert;
                vert.position[0] = pen_x + orig_vert.position[0] * scale + cached.xoffset * scale;
                vert.position[1] = y + orig_vert.position[1] * scale + cached.yoffset * scale;
                vert.color = color.0;
                all_verts.push(vert);
            }
            // Индексы для прямоугольника (6 штук) на 4 вершины
            all_indices.extend_from_slice(&[
                base_idx, base_idx+1, base_idx+2,
                base_idx+1, base_idx+3, base_idx+2,
            ]);
            pen_x += cached.xadvance * scale;
        }
        (all_verts, all_indices)
    }
}

impl Default for FontSystem {
    fn default() -> Self {
        Self::new()
    }
}