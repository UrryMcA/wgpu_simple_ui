use crate::common::vertex::{Vertex};
use crate::common::primitives::{Primitives};
use std::f32::consts::PI;

pub struct DefaultPrimitives;

impl Primitives for DefaultPrimitives {
    fn rect_vertices(&self, x: f32, y: f32, w: f32, h: f32, color: [f32;4]) -> Vec<Vertex> {
        let (x2, y2) = (x + w, y + h);
        vec![
            Vertex { position: [x, y], tex_coord: [0.0, 0.0], color },
            Vertex { position: [x2, y], tex_coord: [1.0, 0.0], color },
            Vertex { position: [x, y2], tex_coord: [0.0, 1.0], color },
            Vertex { position: [x2, y], tex_coord: [1.0, 0.0], color },
            Vertex { position: [x2, y2], tex_coord: [1.0, 1.0], color },
            Vertex { position: [x, y2], tex_coord: [0.0, 1.0], color },
        ]
    }

    fn textured_rect_vertices(&self, x: f32, y: f32, w: f32, h: f32, u0: f32, v0: f32, u1: f32, v1: f32, color: [f32;4]) -> Vec<Vertex> {
        let (x2, y2) = (x + w, y + h);
        vec![
            Vertex { position: [x, y], tex_coord: [u0, v0], color },
            Vertex { position: [x2, y], tex_coord: [u1, v0], color },
            Vertex { position: [x, y2], tex_coord: [u0, v1], color },
            Vertex { position: [x2, y], tex_coord: [u1, v0], color },
            Vertex { position: [x2, y2], tex_coord: [u1, v1], color },
            Vertex { position: [x, y2], tex_coord: [u0, v1], color },
        ]
    }

    fn line_vertices(&self, x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, color: [f32;4]) -> Vec<Vertex> {
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
            Vertex { position: [x, y], tex_coord: [0.0, 0.0], color }
        }).collect();
        vec![verts[0], verts[1], verts[2], verts[1], verts[3], verts[2]]
    }

    fn arc_vertices(&self, cx: f32, cy: f32, r: f32, start: f32, end: f32, thick: f32, color: [f32;4]) -> Vec<Vertex> {
        let mut vertices = Vec::new();
        let step = 0.05;
        let mut angle = start;
        let mut first = true;
        let mut prev = (cx + r * angle.cos(), cy + r * angle.sin());
        while angle <= end {
            let x = cx + r * angle.cos();
            let y = cy + r * angle.sin();
            if !first {
                let line = self.line_vertices(prev.0, prev.1, x, y, thick, color);
                vertices.extend(line);
            }
            prev = (x, y);
            first = false;
            angle += step;
        }
        vertices
    }

    fn filled_arc_sector_vertices(&self, cx: f32, cy: f32, r: f32, start: f32, end: f32, color: [f32;4]) -> Vec<Vertex> {
        let segments = 20;
        let delta = (end - start) / segments as f32;
        let mut verts = vec![Vertex { position: [cx, cy], tex_coord: [0.0, 0.0], color }];
        for i in 0..=segments {
            let angle = start + delta * i as f32;
            let x = cx + r * angle.cos();
            let y = cy + r * angle.sin();
            verts.push(Vertex { position: [x, y], tex_coord: [0.0, 0.0], color });
        }
        let mut triangles = Vec::new();
        for i in 1..verts.len() - 1 {
            triangles.push(verts[0]);
            triangles.push(verts[i]);
            triangles.push(verts[i + 1]);
        }
        triangles
    }

    fn rounded_rect_vertices(&self, x: f32, y: f32, w: f32, h: f32, radius: f32, color: [f32;4]) -> Vec<Vertex> {
        let r = radius.min(w * 0.5).min(h * 0.5);
        let mut vertices = Vec::new();
        if w > 2.0 * r {
            vertices.extend(self.rect_vertices(x + r, y, w - 2.0 * r, h, color));
        }
        if h > 2.0 * r {
            vertices.extend(self.rect_vertices(x, y + r, w, h - 2.0 * r, color));
        }
        let sectors = [
            (x + r, y + r, PI, 1.5 * PI),
            (x + r, y + h - r, 0.5 * PI, PI),
            (x + w - r, y + h - r, 0.0, 0.5 * PI),
            (x + w - r, y + r, 1.5 * PI, 2.0 * PI),
        ];
        for (cx, cy, start, end) in sectors {
            vertices.extend(self.filled_arc_sector_vertices(cx, cy, r, start, end, color));
        }
        vertices
    }

    fn rounded_rect_outline_vertices(&self, x: f32, y: f32, w: f32, h: f32, radius: f32, thickness: f32, color: [f32;4]) -> Vec<Vertex> {
        let r = radius.min(w * 0.5).min(h * 0.5);
        let mut vertices = Vec::new();
        if w > 2.0 * r {
            vertices.extend(self.line_vertices(x + r, y, x + w - r, y, thickness, color));
            vertices.extend(self.line_vertices(x + r, y + h, x + w - r, y + h, thickness, color));
        }
        if h > 2.0 * r {
            vertices.extend(self.line_vertices(x, y + r, x, y + h - r, thickness, color));
            vertices.extend(self.line_vertices(x + w, y + r, x + w, y + h - r, thickness, color));
        }
        vertices.extend(self.arc_vertices(x + r, y + r, r, PI, 1.5 * PI, thickness, color));
        vertices.extend(self.arc_vertices(x + r, y + h - r, r, 0.5 * PI, PI, thickness, color));
        vertices.extend(self.arc_vertices(x + w - r, y + h - r, r, 0.0, 0.5 * PI, thickness, color));
        vertices.extend(self.arc_vertices(x + w - r, y + r, r, 1.5 * PI, 2.0 * PI, thickness, color));
        vertices
    }
}