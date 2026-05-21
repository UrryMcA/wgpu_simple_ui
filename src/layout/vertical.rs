use crate::common::{
    layout_strategy::{LayoutMeasurer, LayoutArranger},
    render_box::RenderBox,
    types::{Alignment, Constraints, LayoutContext, Rect, Size},
};

#[derive(Clone)]
pub struct VerticalLayout {
    pub spacing: f32,
    pub main_alignment: Alignment,
    pub cross_alignment: Alignment,
}

impl VerticalLayout {
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

impl LayoutMeasurer for VerticalLayout {
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
        let total_height = children
            .iter()
            .map(|c| c.size().height)
            .sum::<f32>()
            + self.spacing * (children.len().saturating_sub(1)) as f32;
        let max_width = children
            .iter()
            .map(|c| c.size().width)
            .fold(0.0, f32::max);
        constraints.constrain(Size::new(max_width, total_height))
    }
}

impl LayoutArranger for VerticalLayout {
    fn arrange(&mut self, children: &mut [&mut dyn RenderBox], inner_rect: Rect) -> Vec<Rect> {
        if children.is_empty() {
            return Vec::new();
        }
        let total_children_height: f32 = children.iter().map(|c| c.size().height).sum();
        let total_spacing = self.spacing * (children.len() - 1) as f32;
        let needed_height = total_children_height + total_spacing;

        let start_y = match self.main_alignment {
            Alignment::Start => inner_rect.y,
            Alignment::Center => inner_rect.y + (inner_rect.h - needed_height) / 2.0,
            Alignment::End => inner_rect.y + inner_rect.h - needed_height,
            Alignment::Stretch => inner_rect.y,
        };

        let mut rects = Vec::with_capacity(children.len());
        let mut current_y = start_y;

        for child in children {
            let child_size = child.size();
            let x = match self.cross_alignment {
                Alignment::Start => inner_rect.x,
                Alignment::Center => inner_rect.x + (inner_rect.w - child_size.width) / 2.0,
                Alignment::End => inner_rect.x + inner_rect.w - child_size.width,
                Alignment::Stretch => inner_rect.x,
            };
            let width = if self.cross_alignment == Alignment::Stretch {
                inner_rect.w
            } else {
                child_size.width
            };
            rects.push(Rect::new(
                x,
                current_y,
                width,
                child_size.height,
            ));
            current_y += child_size.height + self.spacing;
        }
        rects
    }
}

impl Default for VerticalLayout {
    fn default() -> Self {
        Self::new()
    }
}