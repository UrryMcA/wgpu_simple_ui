use crate::common::types::{Arc, FilledArcSector, Line, Rect, TexCoords, UColor};

use super::vertex::Vertex;

pub trait Primitives {
    // Залитый цветом прямоугольник
    fn rect_vertices(&self, rect: Rect, color: UColor) -> Vec<Vertex>;

    // Текстурированный прямоугольник (с UV-координатами)
    fn textured_rect_vertices(&self, rect: Rect, tex_coords: TexCoords, color: UColor) -> Vec<Vertex>;

    // Линия заданной толщины
    fn line_vertices(&self, line: Line, color: UColor) -> Vec<Vertex>;

    // Дуга (открытая кривая) заданной толщины
    fn arc_vertices(&self, arc: Arc, color: UColor) -> Vec<Vertex>;

    // Залитый сектор (замкнутая фигура от центра до дуги)
    fn filled_arc_sector_vertices(&self, sector: FilledArcSector, color: UColor) -> Vec<Vertex>;

    // Скруглённый залитый прямоугольник
    fn rounded_rect_vertices(&self,  rect: Rect,  radius: f32, color: UColor) -> Vec<Vertex>;

    // Контур скруглённого прямоугольника
    fn rounded_rect_outline_vertices(&self,rect: Rect,  radius: f32, thickness: f32, color: UColor) -> Vec<Vertex>;

        // ----- новые методы с индексами -----
    fn rect_vertices_indices(&self, rect: Rect, color: UColor) -> (Vec<Vertex>, Vec<u32>);
    fn textured_rect_vertices_indices(&self, rect: Rect, tex_coords: TexCoords, color: UColor) -> (Vec<Vertex>, Vec<u32>);
    fn line_vertices_indices(&self, line: Line, color: UColor) -> (Vec<Vertex>, Vec<u32>);
    fn arc_vertices_indices(&self, arc: Arc, color: UColor) -> (Vec<Vertex>, Vec<u32>);
    fn filled_arc_sector_vertices_indices(&self, sector: FilledArcSector, color: UColor) -> (Vec<Vertex>, Vec<u32>);
    fn rounded_rect_vertices_indices(&self, rect: Rect, radius: f32, color: UColor) -> (Vec<Vertex>, Vec<u32>);
    fn rounded_rect_outline_vertices_indices(&self, rect: Rect, radius: f32, thickness: f32, color: UColor) -> (Vec<Vertex>, Vec<u32>);
    
        // Текстурированный скруглённый прямоугольник с кастомными UV и тинтом
    fn rounded_textured_rect_vertices_indices(
        &self, 
        rect: Rect, 
        radius: f32, 
        tex_coords: TexCoords, 
        tint: UColor
    ) -> (Vec<Vertex>, Vec<u32>);
}