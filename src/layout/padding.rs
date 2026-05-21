use crate::common::{
    layout_strategy::{LayoutMeasurer, LayoutArranger},
    render_box::RenderBox,
    types::{Constraints, LayoutContext, Rect, Size},
};

#[derive(Clone, Copy)]
pub struct PaddingLayout {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

impl PaddingLayout {
    pub fn new(l: f32, r: f32, t: f32, b: f32) -> Self {
        Self {
            left: l,
            right: r,
            top: t,
            bottom: b,
        }
    }
    pub fn all(p: f32) -> Self {
        Self::new(p, p, p, p)
    }
    pub fn symmetric(horizontal: f32, vertical: f32) -> Self {
        Self::new(horizontal, horizontal, vertical, vertical)
    }
}

impl LayoutMeasurer for PaddingLayout {
    fn measure(
        &mut self,
        children: &mut [&mut dyn RenderBox],
        constraints: Constraints,
        ctx: &mut dyn LayoutContext,
    ) -> Size {
        assert_eq!(children.len(), 1, "PaddingLayout requires exactly one child");
        let inner_constraints = Constraints {
            min_width: (constraints.min_width - self.left - self.right).max(0.0),
            max_width: (constraints.max_width - self.left - self.right).max(0.0),
            min_height: (constraints.min_height - self.top - self.bottom).max(0.0),
            max_height: (constraints.max_height - self.top - self.bottom).max(0.0),
        };
        let child_size = children[0].layout(inner_constraints, ctx);
        let total = Size::new(
            child_size.width + self.left + self.right,
            child_size.height + self.top + self.bottom,
        );
        constraints.constrain(total)
    }
}

impl LayoutArranger for PaddingLayout {
    fn arrange(&mut self, children: &mut [&mut dyn RenderBox], inner_rect: Rect) -> Vec<Rect> {
        assert_eq!(children.len(), 1, "PaddingLayout requires exactly one child");
        let child_rect = Rect::new(
            inner_rect.x + self.left,
            inner_rect.y + self.top,
            (inner_rect.w - self.left - self.right).max(0.0),
            (inner_rect.h - self.top - self.bottom).max(0.0),
        );
        vec![child_rect]
    }
}