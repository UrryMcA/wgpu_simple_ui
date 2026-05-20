use super::vertex::Vertex;

pub trait Primitives {
    fn rect_vertices(&self, x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) -> Vec<Vertex>;
    fn textured_rect_vertices(&self, x: f32, y: f32, w: f32, h: f32, u0: f32, v0: f32, u1: f32, v1: f32, color: [f32; 4]) -> Vec<Vertex>;
    fn line_vertices(&self, x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, color: [f32; 4]) -> Vec<Vertex>;
    fn arc_vertices(&self, cx: f32, cy: f32, r: f32, start: f32, end: f32, thick: f32, color: [f32;4]) -> Vec<Vertex>;
    fn filled_arc_sector_vertices(&self, cx: f32, cy: f32, r: f32, start: f32, end: f32, color: [f32;4]) -> Vec<Vertex>;
    fn rounded_rect_vertices(&self, x: f32, y: f32, w: f32, h: f32, radius: f32, color: [f32;4]) -> Vec<Vertex>;
    fn rounded_rect_outline_vertices(&self, x: f32, y: f32, w: f32, h: f32, radius: f32, thickness: f32, color: [f32;4]) -> Vec<Vertex>;
}