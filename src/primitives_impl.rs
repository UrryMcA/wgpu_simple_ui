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
// primitives_impl.rs


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

// ----- НОВЫЕ МЕТОДЫ -----
    fn rect_vertices_indices(&self, rect: Rect, color: UColor) -> (Vec<Vertex>, Vec<u32>) {
        let x = rect.x; let y = rect.y;
        let x2 = x + rect.w; let y2 = y + rect.h;
        let c = color.0;
        let vertices = vec![
            Vertex { position: [x, y], tex_coord: [0.0, 0.0], color: c },
            Vertex { position: [x2, y], tex_coord: [1.0, 0.0], color: c },
            Vertex { position: [x2, y2], tex_coord: [1.0, 1.0], color: c },
            Vertex { position: [x, y2], tex_coord: [0.0, 1.0], color: c },
        ];
        let indices = vec![0, 1, 2, 0, 2, 3];
        (vertices, indices)
    }

    fn textured_rect_vertices_indices(&self, rect: Rect, tex_coords: TexCoords, color: UColor) -> (Vec<Vertex>, Vec<u32>) {
        let x = rect.x; let y = rect.y;
        let x2 = x + rect.w; let y2 = y + rect.h;
        let c = color.0;
        let (u0, v0, u1, v1) = (tex_coords.u0, tex_coords.v0, tex_coords.u1, tex_coords.v1);
        let vertices = vec![
            Vertex { position: [x, y], tex_coord: [u0, v0], color: c },
            Vertex { position: [x2, y], tex_coord: [u1, v0], color: c },
            Vertex { position: [x2, y2], tex_coord: [u1, v1], color: c },
            Vertex { position: [x, y2], tex_coord: [u0, v1], color: c },
        ];
        let indices = vec![0, 1, 2, 0, 2, 3];
        (vertices, indices)
    }

    fn line_vertices_indices(&self, line: Line, color: UColor) -> (Vec<Vertex>, Vec<u32>) {
        let x1 = line.x1; let y1 = line.y1;
        let x2 = line.x2; let y2 = line.y2;
        let thickness = line.thickness;
        let c = color.0;
        let dx = x2 - x1;
        let dy = y2 - y1;
        let len = dx.hypot(dy);
        if len < 1e-6 { return (Vec::new(), Vec::new()); }
        let angle = dy.atan2(dx);
        let cos = angle.cos();
        let sin = angle.sin();
        let half = thickness * 0.5;
        let cx = (x1 + x2) * 0.5;
        let cy = (y1 + y2) * 0.5;
        let local = [
            (-len * 0.5, -half),
            ( len * 0.5, -half),
            ( len * 0.5,  half),
            (-len * 0.5,  half),
        ];
        let vertices: Vec<Vertex> = local.iter().map(|(lx, ly)| {
            let x = cx + lx * cos - ly * sin;
            let y = cy + lx * sin + ly * cos;
            Vertex { position: [x, y], tex_coord: [0.0, 0.0], color: c }
        }).collect();
        let indices = vec![0, 1, 2, 0, 2, 3];
        (vertices, indices)
    }

    fn arc_vertices_indices(&self, arc: Arc, color: UColor) -> (Vec<Vertex>, Vec<u32>) {
        // Для дуги используем старый метод, который возвращает вершины (уже треугольники),
        // и генерируем последовательные индексы.
        let vertices = self.arc_vertices(arc, color);
        let indices = (0..vertices.len() as u32).collect();
        (vertices, indices)
    }

    fn filled_arc_sector_vertices_indices(&self, sector: FilledArcSector, color: UColor) -> (Vec<Vertex>, Vec<u32>) {
        let vertices = self.filled_arc_sector_vertices(sector, color);
        let indices = (0..vertices.len() as u32).collect();
        (vertices, indices)
    }


fn rounded_rect_outline_vertices_indices(
    &self,
    rect: Rect,
    radius: f32,
    thickness: f32,
    color: UColor,
) -> (Vec<Vertex>, Vec<u32>) {
    let x = rect.x;
    let y = rect.y;
    let w = rect.w;
    let h = rect.h;
    let r = radius.min(w * 0.5).min(h * 0.5);
    let c = color.0;
    
    // Внутренний и внешний радиусы
    let half_thick = thickness * 0.5;
    let inner_r = (r - half_thick).max(0.0);
    let outer_r = r + half_thick;
    
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    
    let segments = 8;
    let angle_step = std::f32::consts::FRAC_PI_2 / segments as f32;
    
    // Вспомогательная функция для добавления пары вершин (внутренняя + внешняя)
    let mut add_vertex_pair = |px_inner: f32, py_inner: f32, px_outer: f32, py_outer: f32| {
        let base_idx = vertices.len() as u32;
        
        // Внутренняя вершина
        vertices.push(Vertex {
            position: [px_inner, py_inner],
            tex_coord: [(px_inner - x) / w, (py_inner - y) / h],
            color: c,
        });
        
        // Внешняя вершина
        vertices.push(Vertex {
            position: [px_outer, py_outer],
            tex_coord: [(px_outer - x) / w, (py_outer - y) / h],
            color: c,
        });
        
        base_idx
    };
    
    // Генерируем вершины последовательно по часовой стрелке
    
    // 1. Верхняя сторона (слева направо)
    add_vertex_pair(x + inner_r, y + half_thick, x + outer_r, y - half_thick);
    
    // 2. Верхний правый угол
    let corner_cx = x + w - r;
    let corner_cy = y + r;
    for i in 0..=segments {
        let angle = -std::f32::consts::FRAC_PI_2 + angle_step * i as f32;
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        
        let px_inner = corner_cx + inner_r * cos_a;
        let py_inner = corner_cy + inner_r * sin_a;
        let px_outer = corner_cx + outer_r * cos_a;
        let py_outer = corner_cy + outer_r * sin_a;
        
        add_vertex_pair(px_inner, py_inner, px_outer, py_outer);
    }
    
    // 3. Правая сторона (сверху вниз)
    add_vertex_pair(x + w - half_thick, y + h - inner_r, x + w + half_thick, y + h - outer_r);
    
    // 4. Нижний правый угол
    let corner_cx = x + w - r;
    let corner_cy = y + h - r;
    for i in 0..=segments {
        let angle = angle_step * i as f32;
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        
        let px_inner = corner_cx + inner_r * cos_a;
        let py_inner = corner_cy + inner_r * sin_a;
        let px_outer = corner_cx + outer_r * cos_a;
        let py_outer = corner_cy + outer_r * sin_a;
        
        add_vertex_pair(px_inner, py_inner, px_outer, py_outer);
    }
    
    // 5. Нижняя сторона (справа налево)
    add_vertex_pair(x + w - inner_r, y + h - half_thick, x + w - outer_r, y + h + half_thick);
    
    // 6. Нижний левый угол
    let corner_cx = x + r;
    let corner_cy = y + h - r;
    for i in 0..=segments {
        let angle = std::f32::consts::FRAC_PI_2 + angle_step * i as f32;
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        
        let px_inner = corner_cx + inner_r * cos_a;
        let py_inner = corner_cy + inner_r * sin_a;
        let px_outer = corner_cx + outer_r * cos_a;
        let py_outer = corner_cy + outer_r * sin_a;
        
        add_vertex_pair(px_inner, py_inner, px_outer, py_outer);
    }
    
    // 7. Левая сторона (снизу вверх)
    add_vertex_pair(x + half_thick, y + h - inner_r, x - half_thick, y + h - outer_r);
    
    // 8. Верхний левый угол
    let corner_cx = x + r;
    let corner_cy = y + r;
    for i in 0..=segments {
        let angle = std::f32::consts::PI + angle_step * i as f32;
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        
        let px_inner = corner_cx + inner_r * cos_a;
        let py_inner = corner_cy + inner_r * sin_a;
        let px_outer = corner_cx + outer_r * cos_a;
        let py_outer = corner_cy + outer_r * sin_a;
        
        add_vertex_pair(px_inner, py_inner, px_outer, py_outer);
    }
    
    // Генерируем индексы (соединяем внутренний и внешний периметры)
    let pair_count = vertices.len() as u32 / 2;
    for i in 0..pair_count {
        let current_inner = i * 2;
        let current_outer = i * 2 + 1;
        let next_inner = if i + 1 == pair_count { 0 } else { (i + 1) * 2 };
        let next_outer = if i + 1 == pair_count { 1 } else { (i + 1) * 2 + 1 };
        
        // Первый треугольник: current_inner -> current_outer -> next_inner
        indices.push(current_inner);
        indices.push(current_outer);
        indices.push(next_inner);
        
        // Второй треугольник: next_inner -> current_outer -> next_outer
        indices.push(next_inner);
        indices.push(current_outer);
        indices.push(next_outer);
    }
    
    (vertices, indices)
}

fn rounded_rect_vertices_indices(&self, rect: Rect, radius: f32, color: UColor) -> (Vec<Vertex>, Vec<u32>) {
    let x = rect.x;
    let y = rect.y;
    let w = rect.w;
    let h = rect.h;
    let r = radius.min(w * 0.5).min(h * 0.5);
    let c = color.0;
    
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    
    // Центральная вершина (индекс 0)
    let center_x = x + w * 0.5;
    let center_y = y + h * 0.5;
    vertices.push(Vertex {
        position: [center_x, center_y],
        tex_coord: [0.5, 0.5],
        color: c,
    });
    
    // Количество сегментов для каждого угла (90 градусов)
    let segments = 8;
    let angle_step = std::f32::consts::FRAC_PI_2 / segments as f32;
    
    // Генерируем вершины периметра ПОСЛЕДОВАТЕЛЬНО по часовой стрелке
    
    // 1. Верхняя сторона (слева направо, после верхнего левого скругления)
    vertices.push(Vertex {
        position: [x + r, y],
        tex_coord: [r / w, 0.0],
        color: c,
    });
    
    // 2. Верхний правый угол (от 270° до 360°, т.е. от -90° до 0°)
    let corner_cx = x + w - r;
    let corner_cy = y + r;
    for i in 0..=segments {
        let angle = -std::f32::consts::FRAC_PI_2 + angle_step * i as f32;
        let px = corner_cx + r * angle.cos();
        let py = corner_cy + r * angle.sin();
        vertices.push(Vertex {
            position: [px, py],
            tex_coord: [(px - x) / w, (py - y) / h],
            color: c,
        });
    }
    
    // 3. Правая сторона (сверху вниз)
    vertices.push(Vertex {
        position: [x + w, y + h - r],
        tex_coord: [1.0, (h - r) / h],
        color: c,
    });
    
    // 4. Нижний правый угол (от 0° до 90°)
    let corner_cx = x + w - r;
    let corner_cy = y + h - r;
    for i in 0..=segments {
        let angle = angle_step * i as f32;
        let px = corner_cx + r * angle.cos();
        let py = corner_cy + r * angle.sin();
        vertices.push(Vertex {
            position: [px, py],
            tex_coord: [(px - x) / w, (py - y) / h],
            color: c,
        });
    }
    
    // 5. Нижняя сторона (справа налево)
    vertices.push(Vertex {
        position: [x + r, y + h],
        tex_coord: [r / w, 1.0],
        color: c,
    });
    
    // 6. Нижний левый угол (от 90° до 180°)
    let corner_cx = x + r;
    let corner_cy = y + h - r;
    for i in 0..=segments {
        let angle = std::f32::consts::FRAC_PI_2 + angle_step * i as f32;
        let px = corner_cx + r * angle.cos();
        let py = corner_cy + r * angle.sin();
        vertices.push(Vertex {
            position: [px, py],
            tex_coord: [(px - x) / w, (py - y) / h],
            color: c,
        });
    }
    
    // 7. Левая сторона (снизу вверх)
    vertices.push(Vertex {
        position: [x, y + r],
        tex_coord: [0.0, r / h],
        color: c,
    });
    
    // 8. Верхний левый угол (от 180° до 270°)
    let corner_cx = x + r;
    let corner_cy = y + r;
    for i in 0..=segments {
        let angle = std::f32::consts::PI + angle_step * i as f32;
        let px = corner_cx + r * angle.cos();
        let py = corner_cy + r * angle.sin();
        vertices.push(Vertex {
            position: [px, py],
            tex_coord: [(px - x) / w, (py - y) / h],
            color: c,
        });
    }
    
    // Генерируем индексы (треугольники-веер от центра)
    let perimeter_count = vertices.len() as u32 - 1; // минус центральная вершина
    for i in 0..perimeter_count {
        let current = i + 1;
        let next = if i + 1 == perimeter_count { 1 } else { i + 2 };
        indices.push(0); // центр
        indices.push(current);
        indices.push(next);
    }
    
    (vertices, indices)
}

    fn rounded_textured_rect_vertices_indices(
        &self,
        rect: Rect,
        radius: f32,
        tex_coords: TexCoords,
        tint: UColor,
    ) -> (Vec<Vertex>, Vec<u32>) {
        let x = rect.x;
        let y = rect.y;
        let w = rect.w.max(1.0);
        let h = rect.h.max(1.0);
        let r = radius.min(w * 0.5).min(h * 0.5);
        let c = tint.0;

        // Билинейная интерполяция UV
        let du = (tex_coords.u1 - tex_coords.u0) / w;
        let dv = (tex_coords.v1 - tex_coords.v0) / h;

        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        // Центр (индекс 0)
        let cx = x + w * 0.5;
        let cy = y + h * 0.5;
        vertices.push(Vertex {
            position: [cx, cy],
            tex_coord: [tex_coords.u0 + du * (w * 0.5), tex_coords.v0 + dv * (h * 0.5)],
            color: c,
        });

        let segments = 8;
        let angle_step = std::f32::consts::FRAC_PI_2 / segments as f32;

        let mut push_v = |px: f32, py: f32| {
            vertices.push(Vertex {
                position: [px, py],
                tex_coord: [tex_coords.u0 + (px - x) * du, tex_coords.v0 + (py - y) * dv],
                color: c,
            });
        };

        // Периметр по часовой стрелке
        push_v(x + r, y);
        for i in 0..=segments {
            let a = -std::f32::consts::FRAC_PI_2 + angle_step * i as f32;
            push_v(x + w - r + r * a.cos(), y + r + r * a.sin());
        }
        push_v(x + w, y + h - r);
        for i in 0..=segments {
            let a = angle_step * i as f32;
            push_v(x + w - r + r * a.cos(), y + h - r + r * a.sin());
        }
        push_v(x + r, y + h);
        for i in 0..=segments {
            let a = std::f32::consts::FRAC_PI_2 + angle_step * i as f32;
            push_v(x + r + r * a.cos(), y + h - r + r * a.sin());
        }
        push_v(x, y + r);
        for i in 0..=segments {
            let a = std::f32::consts::PI + angle_step * i as f32;
            push_v(x + r + r * a.cos(), y + r + r * a.sin());
        }

        // Triangle fan
        let perim = vertices.len() as u32 - 1;
        for i in 0..perim {
            let cur = i + 1;
            let next = if i + 1 == perim { 1 } else { i + 2 };
            indices.push(0);
            indices.push(cur);
            indices.push(next);
        }

        (vertices, indices)
    }   
}