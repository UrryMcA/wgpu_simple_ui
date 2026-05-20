use super::vertex::Vertex;
use super::primitives::Primitives;

#[derive(Debug, Clone, Copy)]
pub struct GlyphInfo {
    pub width: f32,
    pub height: f32,
    pub u0: f32, pub v0: f32,
    pub u1: f32, pub v1: f32,
    pub xoffset: f32,
    pub yoffset: f32,
    pub xadvance: f32,
}

pub trait BitmapFont {
    fn texture_id(&self) -> u64;
    fn line_height(&self) -> f32;
    fn get_glyph(&self, ch: char) -> Option<GlyphInfo>;
}

pub fn generate_text_vertices(
    font: &dyn BitmapFont,
    text: &str,
    x: f32, y: f32,
    scale: f32,
    color: [f32;4],
    primitives: &dyn Primitives,
) -> Vec<Vertex> {
    let mut vertices = Vec::new();
    let mut pen_x = x;
    for ch in text.chars() {
        if let Some(g) = font.get_glyph(ch) {
            let w = g.width * scale;
            let h = g.height * scale;
            let verts = primitives.textured_rect_vertices(
                pen_x + g.xoffset * scale,
                y + g.yoffset * scale,
                w, h,
                g.u0, g.v0, g.u1, g.v1,
                color,
            );
            vertices.extend(verts);
            pen_x += g.xadvance * scale;
        }
    }
    vertices
}