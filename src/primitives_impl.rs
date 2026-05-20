use crate::common::vertex::Vertex;
use crate::common::primitives::Primitives;
use crate::common::types::{Rect, TexCoords, UColor, Line, Arc, FilledArcSector};
use std::f32::consts::PI;

pub struct DefaultPrimitives;

impl Primitives for DefaultPrimitives {
    fn rect_vertices(&self, rect: Rect, color: UColor) -> Vec<Vertex> {
        let x = rect.x;
        let y = rect.y;
        let w = rect.w;
        let h = rect.h;
        let (x2, y2) = (x + w, y + h);
        let color_arr = color.0;
        vec![
            Vertex { position: [x, y], tex_coord: [0.0, 0.0], color: color_arr },
            Vertex { position: [x2, y], tex_coord: [1.0, 0.0], color: color_arr },
            Vertex { position: [x, y2], tex_coord: [0.0, 1.0], color: color_arr },
            Vertex { position: [x2, y], tex_coord: [1.0, 0.0], color: color_arr },
            Vertex { position: [x2, y2], tex_coord: [1.0, 1.0], color: color_arr },
            Vertex { position: [x, y2], tex_coord: [0.0, 1.0], color: color_arr },
        ]
    }

    fn textured_rect_vertices(&self, rect: Rect, tex_coords: TexCoords, color: UColor) -> Vec<Vertex> {
        let x = rect.x;
        let y = rect.y;
        let w = rect.w;
        let h = rect.h;
        let (x2, y2) = (x + w, y + h);
        let (u0, v0, u1, v1) = (tex_coords.u0, tex_coords.v0, tex_coords.u1, tex_coords.v1);
        let color_arr = color.0;
        vec![
            Vertex { position: [x, y], tex_coord: [u0, v0], color: color_arr },
            Vertex { position: [x2, y], tex_coord: [u1, v0], color: color_arr },
            Vertex { position: [x, y2], tex_coord: [u0, v1], color: color_arr },
            Vertex { position: [x2, y], tex_coord: [u1, v0], color: color_arr },
            Vertex { position: [x2, y2], tex_coord: [u1, v1], color: color_arr },
            Vertex { position: [x, y2], tex_coord: [u0, v1], color: color_arr },
        ]
    }

    fn line_vertices(&self, line: Line, color: UColor) -> Vec<Vertex> {
        let x1 = line.x1; let y1 = line.y1;
        let x2 = line.x2; let y2 = line.y2;
        let thickness = line.thickness;
        let color_arr = color.0;
        let dx = x2 - x1;
        let dy = y2 - y1;
        let len = dx.hypot(dy);
        if len < 1e-6 { return vec![]; }
        let angle = dy.atan2(dx);
        let cos = angle.cos();
        let sin = angle.sin();
        let half = thickness * 0.5;
        let cx = (x1 + x2) * 0.5;
        let cy = (y1 + y2) * 0.5;
        let local = [
            (-len * 0.5, -half),
            ( len * 0.5, -half),
            (-len * 0.5,  half),
            ( len * 0.5,  half),
        ];
        let verts: Vec<Vertex> = local.iter().map(|(lx, ly)| {
            let x = cx + lx * cos - ly * sin;
            let y = cy + lx * sin + ly * cos;
            Vertex { position: [x, y], tex_coord: [0.0, 0.0], color: color_arr }
        }).collect();
        vec![verts[0], verts[1], verts[2], verts[1], verts[3], verts[2]]
    }

    fn arc_vertices(&self, arc: Arc, color: UColor) -> Vec<Vertex> {
        let cx = arc.cx; let cy = arc.cy;
        let r = arc.r;
        let start = arc.start_angle;
        let end = arc.end_angle;
        let thick = arc.thickness;
        let color_arr = color.0;
        let mut vertices = Vec::new();
        let step = 0.05;
        let mut angle = start;
        let mut first = true;
        let mut prev = (cx + r * angle.cos(), cy + r * angle.sin());
        while angle <= end {
            let x = cx + r * angle.cos();
            let y = cy + r * angle.sin();
            if !first {
                let line = Line {
                    x1: prev.0, y1: prev.1,
                    x2: x, y2: y,
                    thickness: thick,
                };
                vertices.extend(self.line_vertices(line, UColor(color_arr)));
            }
            prev = (x, y);
            first = false;
            angle += step;
        }
        vertices
    }

    fn filled_arc_sector_vertices(&self, sector: FilledArcSector, color: UColor) -> Vec<Vertex> {
        let cx = sector.cx; let cy = sector.cy;
        let r = sector.r;
        let start = sector.start_angle;
        let end = sector.end_angle;
        let color_arr = color.0;
        let segments = 20;
        let delta = (end - start) / segments as f32;
        let mut verts = vec![Vertex { position: [cx, cy], tex_coord: [0.0, 0.0], color: color_arr }];
        for i in 0..=segments {
            let angle = start + delta * i as f32;
            let x = cx + r * angle.cos();
            let y = cy + r * angle.sin();
            verts.push(Vertex { position: [x, y], tex_coord: [0.0, 0.0], color: color_arr });
        }
        let mut triangles = Vec::new();
        for i in 1..verts.len() - 1 {
            triangles.push(verts[0]);
            triangles.push(verts[i]);
            triangles.push(verts[i + 1]);
        }
        triangles
    }

    fn rounded_rect_vertices(&self, rect: Rect, radius: f32, color: UColor) -> Vec<Vertex> {
        let x = rect.x; let y = rect.y;
        let w = rect.w; let h = rect.h;
        let r = radius.min(w * 0.5).min(h * 0.5);
        let color_arr = color.0;
        let mut vertices = Vec::new();
        if w > 2.0 * r {
            let inner_rect = Rect::new(x + r, y, w - 2.0 * r, h);
            vertices.extend(self.rect_vertices(inner_rect, UColor(color_arr)));
        }
        if h > 2.0 * r {
            let inner_rect = Rect::new(x, y + r, w, h - 2.0 * r);
            vertices.extend(self.rect_vertices(inner_rect, UColor(color_arr)));
        }
        let sectors = [
            (x + r, y + r, PI, 1.5 * PI),
            (x + r, y + h - r, 0.5 * PI, PI),
            (x + w - r, y + h - r, 0.0, 0.5 * PI),
            (x + w - r, y + r, 1.5 * PI, 2.0 * PI),
        ];
        for (cx, cy, start, end) in sectors {
            let sector = FilledArcSector::new(cx, cy, r, start, end);
            vertices.extend(self.filled_arc_sector_vertices(sector, UColor(color_arr)));
        }
        vertices
    }

    fn rounded_rect_outline_vertices(&self, rect: Rect, radius: f32, thickness: f32, color: UColor) -> Vec<Vertex> {
        let x = rect.x; let y = rect.y;
        let w = rect.w; let h = rect.h;
        let r = radius.min(w * 0.5).min(h * 0.5);
        let color_arr = color.0;
        let mut vertices = Vec::new();
        if w > 2.0 * r {
            let top_line = Line::new(x + r, y, x + w - r, y, thickness);
            let bottom_line = Line::new(x + r, y + h, x + w - r, y + h, thickness);
            vertices.extend(self.line_vertices(top_line, UColor(color_arr)));
            vertices.extend(self.line_vertices(bottom_line, UColor(color_arr)));
        }
        if h > 2.0 * r {
            let left_line = Line::new(x, y + r, x, y + h - r, thickness);
            let right_line = Line::new(x + w, y + r, x + w, y + h - r, thickness);
            vertices.extend(self.line_vertices(left_line, UColor(color_arr)));
            vertices.extend(self.line_vertices(right_line, UColor(color_arr)));
        }
        let arcs = [
            (x + r, y + r, r, PI, 1.5 * PI),
            (x + r, y + h - r, r, 0.5 * PI, PI),
            (x + w - r, y + h - r, r, 0.0, 0.5 * PI),
            (x + w - r, y + r, r, 1.5 * PI, 2.0 * PI),
        ];
        for (cx, cy, rad, start, end) in arcs {
            let arc = Arc::new(cx, cy, rad, start, end, thickness);
            vertices.extend(self.arc_vertices(arc, UColor(color_arr)));
        }
        vertices
    }
}