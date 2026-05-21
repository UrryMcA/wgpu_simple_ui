use crate::common::{
    layout_strategy::{LayoutMeasurer, LayoutArranger},
    render_box::RenderBox,
    types::{Constraints, LayoutContext, Rect, Size, Point},
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
    fn measure(
        &mut self,
        children: &mut [&mut dyn RenderBox],
        constraints: Constraints,
        ctx: &mut dyn LayoutContext,
    ) -> Size {
        let loose = Constraints::loose();
        for child in children.iter_mut() {
            child.layout(loose, ctx);
        }
        // Находим максимальную ширину и высоту среди всех детей
        let max_child_width = children.iter().map(|c| c.size().width).fold(0.0, f32::max);
        let max_child_height = children.iter().map(|c| c.size().height).fold(0.0, f32::max);
        let total_width = self.cols as f32 * max_child_width + (self.cols - 1) as f32 * self.spacing_x;
        let total_height = self.rows as f32 * max_child_height + (self.rows - 1) as f32 * self.spacing_y;
        constraints.constrain(Size::new(total_width, total_height))
    }
}

impl LayoutArranger for GridLayout {
    fn arrange(&mut self, children: &mut [&mut dyn RenderBox], inner_rect: Rect) -> Vec<Rect> {
        let mut rects = Vec::with_capacity(children.len());
        let cell_width = inner_rect.w / self.cols as f32;
        let cell_height = inner_rect.h / self.rows as f32;
        for (idx, child) in children.iter().enumerate() {
            let row = idx / self.cols;
            let col = idx % self.cols;
            let x = inner_rect.x + col as f32 * (cell_width + self.spacing_x);
            let y = inner_rect.y + row as f32 * (cell_height + self.spacing_y);
            rects.push(Rect::new(x, y, cell_width, cell_height));
        }
        rects
    }
}