use crate::common::types::{Arc, FilledArcSector, Line, Rect, TexCoords, UColor};

use super::vertex::Vertex;

pub trait Primitives {
    /// Залитый цветом прямоугольник
    fn rect_vertices(&self, rect: Rect, color: UColor) -> Vec<Vertex>;

    /// Текстурированный прямоугольник (с UV-координатами)
    fn textured_rect_vertices(&self, rect: Rect, tex_coords: TexCoords, color: UColor) -> Vec<Vertex>;

    /// Линия заданной толщины
    fn line_vertices(&self, line: Line, color: UColor) -> Vec<Vertex>;

    /// Дуга (открытая кривая) заданной толщины
    fn arc_vertices(&self, arc: Arc, color: UColor) -> Vec<Vertex>;

    /// Залитый сектор (замкнутая фигура от центра до дуги)
    fn filled_arc_sector_vertices(&self, sector: FilledArcSector, color: UColor) -> Vec<Vertex>;

    /// Скруглённый залитый прямоугольник
    fn rounded_rect_vertices(&self,  rect: Rect,  radius: f32, color: UColor) -> Vec<Vertex>;

    /// Контур скруглённого прямоугольника
    fn rounded_rect_outline_vertices(&self,rect: Rect,  radius: f32, thickness: f32, color: UColor) -> Vec<Vertex>;
}