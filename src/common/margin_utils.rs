// src/common/margin_utils.rs
use crate::common::render_box::RenderBox;
use crate::common::types::{Constraints, EdgeInsets, Point, Size};

/// Корректирует ограничения, вычитая margin из max/min размеров.
pub fn adjust_constraints_for_margin(constraints: Constraints, margin: EdgeInsets) -> Constraints {
    Constraints {
        min_width: (constraints.min_width - margin.left - margin.right).max(0.0),
        max_width: (constraints.max_width - margin.left - margin.right).max(0.0),
        min_height: (constraints.min_height - margin.top - margin.bottom).max(0.0),
        max_height: (constraints.max_height - margin.top - margin.bottom).max(0.0),
    }
}

/// Применяет margin к позиции (сдвигает на left, top).
pub fn apply_margin_to_position(point: Point, margin: EdgeInsets) -> Point {
    Point::new(point.x + margin.left, point.y + margin.top)
}

/// Корректирует размер для случая растяжения, вычитая margin.
pub fn adjust_size_for_margin(size: Size, margin: EdgeInsets) -> Size {
    Size::new(
        (size.width - margin.left - margin.right).max(0.0),
        (size.height - margin.top - margin.bottom).max(0.0),
    )
}

/// Вспомогательные функции для суммирования размеров детей с учётом margin.
pub struct MarginAccumulator;

impl MarginAccumulator {
    /// Для вертикального расположения: возвращает (max_width, total_height).
    pub fn vertical_sum(children: &[&dyn RenderBox]) -> (f32, f32) {
        let mut total_height: f32 = 0.0;
        let mut max_width: f32 = 0.0;
        for child in children {
            let size = child.size();
            let margin = child.margin();
            total_height += size.height + margin.top + margin.bottom;
            max_width = max_width.max(size.width + margin.left + margin.right);
        }
        (max_width, total_height)
    }

    /// Для горизонтального расположения: возвращает (total_width, max_height).
    pub fn horizontal_sum(children: &[&dyn RenderBox]) -> (f32, f32) {
        let mut total_width: f32 = 0.0;
        let mut max_height: f32 = 0.0;
        for child in children {
            let size = child.size();
            let margin = child.margin();
            total_width += size.width + margin.left + margin.right;
            max_height = max_height.max(size.height + margin.top + margin.bottom);
        }
        (total_width, max_height)
    }
}