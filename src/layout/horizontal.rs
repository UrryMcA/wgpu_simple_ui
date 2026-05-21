use crate::common::{
    layout_strategy::{LayoutMeasurer, LayoutArranger},
    render_box::RenderBox,
    types::{Alignment, Constraints, LayoutContext, Rect, Size},
};

#[derive(Clone)]
pub struct HorizontalLayout {
    pub spacing: f32,
    pub main_alignment: Alignment,
    pub cross_alignment: Alignment,
}

impl HorizontalLayout {
    pub fn new() -> Self {
        Self {
            spacing: 0.0,
            main_alignment: Alignment::Start,
            cross_alignment: Alignment::Stretch,
        }
    }
    pub fn with_spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }
    pub fn with_main_alignment(mut self, a: Alignment) -> Self {
        self.main_alignment = a;
        self
    }
    pub fn with_cross_alignment(mut self, a: Alignment) -> Self {
        self.cross_alignment = a;
        self
    }
}

impl LayoutMeasurer for HorizontalLayout {
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
        let total_width = children
            .iter()
            .map(|c| c.size().width)
            .sum::<f32>()
            + self.spacing * (children.len().saturating_sub(1)) as f32;
        let max_height = children
            .iter()
            .map(|c| c.size().height)
            .fold(0.0, f32::max);
        constraints.constrain(Size::new(total_width, max_height))
    }
}

impl LayoutArranger for HorizontalLayout {
    fn arrange(&mut self, children: &mut [&mut dyn RenderBox], inner_rect: Rect) -> Vec<Rect> {
        if children.is_empty() {
            return Vec::new();
        }
        let total_children_width: f32 = children.iter().map(|c| c.size().width).sum();
        let total_spacing = self.spacing * (children.len() - 1) as f32;
        let needed_width = total_children_width + total_spacing;

        let start_x = match self.main_alignment {
            Alignment::Start => inner_rect.x,
            Alignment::Center => inner_rect.x + (inner_rect.w - needed_width) / 2.0,
            Alignment::End => inner_rect.x + inner_rect.w - needed_width,
            Alignment::Stretch => inner_rect.x,
        };

        let mut rects = Vec::with_capacity(children.len());
        let mut current_x = start_x;

        for child in children {
            let child_size = child.size();
            let y = match self.cross_alignment {
                Alignment::Start => inner_rect.y,
                Alignment::Center => inner_rect.y + (inner_rect.h - child_size.height) / 2.0,
                Alignment::End => inner_rect.y + inner_rect.h - child_size.height,
                Alignment::Stretch => inner_rect.y,
            };
            let height = if self.cross_alignment == Alignment::Stretch {
                inner_rect.h
            } else {
                child_size.height
            };
            rects.push(Rect::new(
                current_x,
                y,
                child_size.width,
                height,
            ));
            current_x += child_size.width + self.spacing;
        }
        rects
    }
}

impl Default for HorizontalLayout {
    fn default() -> Self {
        Self::new()
    }
}