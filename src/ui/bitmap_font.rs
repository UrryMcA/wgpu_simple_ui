use crate::common::{Primitives, Vertex, types::{BitmapFont, Rect, TexCoords, UColor}};

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

            let rect = Rect::new( 
                pen_x + g.xoffset * scale,
                 y + g.yoffset * scale, w, h);
            let tex = TexCoords { u0:g.u0, v0:g.v0, u1:g.u1, v1:g.v1};
            let col = UColor(color);
            let verts = primitives.textured_rect_vertices(rect, tex, col);
/*
            let verts = primitives.textured_rect_vertices(
                pen_x + g.xoffset * scale,
                y + g.yoffset * scale,
                w, h,
                g.u0, g.v0, g.u1, g.v1,
                color,
            );
             */
            vertices.extend(verts);
            pen_x += g.xadvance * scale;
        }
    }
    vertices
}