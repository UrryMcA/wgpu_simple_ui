use crate::common::{
    layout_strategy::{LayoutMeasurer, LayoutArranger},
    render_box::RenderBox,
    types::{Constraints, LayoutContext, Rect, Size},
};

#[derive(Clone)]
pub struct GridLayout {
    pub cols: usize,
    pub rows: usize,
    pub spacing_x: f32,
    pub spacing_y: f32,
}

impl GridLayout {
    pub fn new(cols: usize, rows: usize, spacing_x: f32, spacing_y: f32) -> Self {
        Self { cols, rows, spacing_x, spacing_y }
    }
}

impl LayoutMeasurer for GridLayout {
    fn measure(&mut self, children: &mut [&mut dyn RenderBox], constraints: Constraints, ctx: &mut dyn LayoutContext) -> Size {
        let loose = Constraints::loose();
        for child in children.iter_mut() {
            child.layout(loose, ctx);
        }
        let mut max_child_width: f32 = 0.0;
        let mut max_child_height: f32 = 0.0;
        for child in children.iter() {
            let margin = child.margin();
            let w = child.size().width + margin.left + margin.right;
            let h = child.size().height + margin.top + margin.bottom;
            max_child_width = max_child_width.max(w);
            max_child_height = max_child_height.max(h);
        }
        let total_width = self.cols as f32 * max_child_width + (self.cols - 1) as f32 * self.spacing_x;
        let total_height = self.rows as f32 * max_child_height + (self.rows - 1) as f32 * self.spacing_y;
        constraints.constrain(Size::new(total_width, total_height))
    }
}

impl LayoutArranger for GridLayout {
    fn arrange(&mut self, children: &mut [&mut dyn RenderBox], inner_rect: Rect) -> Vec<Rect> {
        let mut rects = Vec::with_capacity(children.len());
        // Найдём максимальный размер среди детей с учётом margin
        let mut max_child_width: f32 = 0.0;
        let mut max_child_height: f32 = 0.0;
        for child in children.iter() {
            let margin = child.margin();
            let w = child.size().width + margin.left + margin.right;
            let h = child.size().height + margin.top + margin.bottom;
            max_child_width = max_child_width.max(w);
            max_child_height = max_child_height.max(h);
        }
        let cell_width = max_child_width;
        let cell_height = max_child_height;
        // Если inner_rect задаёт другую область, то можно масштабировать, но упростим
        let start_x = inner_rect.x;
        let start_y = inner_rect.y;
        for (idx, child) in children.iter().enumerate() {
            let row = idx / self.cols;
            let col = idx % self.cols;
            let margin = child.margin();
            let x = start_x + col as f32 * (cell_width + self.spacing_x) + margin.left;
            let y = start_y + row as f32 * (cell_height + self.spacing_y) + margin.top;
            let width = (cell_width - margin.left - margin.right).max(0.0);
            let height = (cell_height - margin.top - margin.bottom).max(0.0);
            rects.push(Rect::new(x, y, width, height));
        }
        rects
    }
}