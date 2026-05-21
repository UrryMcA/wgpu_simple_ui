// src/ui/layout.rs
use crate::common::types::{Rect, EdgeInsets, Alignment};

/// Вычисляет позиции дочерних виджетов в вертикальном контейнере
pub fn layout_vertical(
    inner_bounds: Rect,
    children: &[(f32, f32, EdgeInsets)], // (min_width, min_height, margin)
    spacing: f32,
    alignment: Alignment,
) -> Vec<Rect> {
    let mut y = inner_bounds.y;
    let mut result = Vec::new();
    for (w, h, margin) in children {
        let x = match alignment {
            Alignment::Start => inner_bounds.x + margin.left,
            Alignment::Center => inner_bounds.x + (inner_bounds.w - w) / 2.0,
            Alignment::End => inner_bounds.x + inner_bounds.w - w - margin.right,
            Alignment::Stretch => inner_bounds.x + margin.left,
        };
        let width = if let Alignment::Stretch = alignment {
            inner_bounds.w - margin.left - margin.right
        } else {
            *w
        };
        let rect = Rect {
            x,
            y: y + margin.top,
            w: width,
            h: *h,
        };
        result.push(rect);
        y += margin.top + *h + margin.bottom + spacing;
    }
    let total_height: f32 = result.iter().map(|r| r.h).sum::<f32>() + spacing * (children.len().saturating_sub(1)) as f32;
    let offset = match alignment {
        Alignment::Center => (inner_bounds.h - total_height) / 2.0,
        Alignment::End => inner_bounds.h - total_height,
        _ => 0.0,
    };
    for rect in &mut result {
        rect.y += offset;
    }
    result
}

/// Вычисляет позиции дочерних виджетов в горизонтальном контейнере
pub fn layout_horizontal(
    inner_bounds: Rect,
    children: &[(f32, f32, EdgeInsets)],
    spacing: f32,
    alignment: Alignment,
) -> Vec<Rect> {
    let mut x = inner_bounds.x;
    let mut result = Vec::new();
    for (w, h, margin) in children {
        let y = match alignment {
            Alignment::Start => inner_bounds.y + margin.top,
            Alignment::Center => inner_bounds.y + (inner_bounds.h - h) / 2.0,
            Alignment::End => inner_bounds.y + inner_bounds.h - h - margin.bottom,
            Alignment::Stretch => inner_bounds.y + margin.top,
        };
        let height = if let Alignment::Stretch = alignment {
            inner_bounds.h - margin.top - margin.bottom
        } else {
            *h
        };
        let rect = Rect {
            x: x + margin.left,
            y,
            w: *w,
            h: height,
        };
        result.push(rect);
        x += margin.left + *w + margin.right + spacing;
    }
    let total_width: f32 = result.iter().map(|r| r.w).sum::<f32>() + spacing * (children.len().saturating_sub(1)) as f32;
    let offset = match alignment {
        Alignment::Center => (inner_bounds.w - total_width) / 2.0,
        Alignment::End => inner_bounds.w - total_width,
        _ => 0.0,
    };
    for rect in &mut result {
        rect.x += offset;
    }
    result
}

/// Вычисляет позиции дочерних виджетов в сетке (grid)
pub fn layout_grid(
    inner_bounds: Rect,
    children: &[(f32, f32, EdgeInsets)],
    cols: usize,
    rows: usize,
    spacing_x: f32,
    spacing_y: f32,
) -> Vec<Rect> {
    let mut result = Vec::new();
    let cell_width = (inner_bounds.w - (cols - 1) as f32 * spacing_x) / cols as f32;
    let cell_height = (inner_bounds.h - (rows - 1) as f32 * spacing_y) / rows as f32;
    for (idx, (w, h, margin)) in children.iter().enumerate() {
        if idx >= cols * rows {
            break;
        }
        let row = idx / cols;
        let col = idx % cols;
        let cell_x = inner_bounds.x + col as f32 * (cell_width + spacing_x);
        let cell_y = inner_bounds.y + row as f32 * (cell_height + spacing_y);
        let x = cell_x + (cell_width - w) / 2.0;
        let y = cell_y + (cell_height - h) / 2.0;
        result.push(Rect {
            x: x + margin.left,
            y: y + margin.top,
            w: *w,
            h: *h,
        });
    }
    result
}
