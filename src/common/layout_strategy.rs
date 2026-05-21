use crate::common::render_box::RenderBox;
use crate::common::types::{Constraints, LayoutContext, Rect, Size};

pub trait LayoutMeasurer {
    fn measure(
        &mut self,
        children: &mut [&mut dyn RenderBox],
        constraints: Constraints,
        ctx: &mut dyn LayoutContext,
    ) -> Size;
}

pub trait LayoutArranger {
    fn arrange(
        &mut self,
        children: &mut [&mut dyn RenderBox],
        inner_rect: Rect,
    ) -> Vec<Rect>;
}

// Объединяющий трейт (опционально)
pub trait LayoutStrategy: LayoutMeasurer + LayoutArranger {}
impl<T: LayoutMeasurer + LayoutArranger> LayoutStrategy for T {}