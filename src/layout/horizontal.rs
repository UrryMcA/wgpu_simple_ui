use crate::common::{
    layout_strategy::{LayoutArranger, LayoutMeasurer}, margin_utils::MarginAccumulator, render_box::RenderBox, types::{Alignment, Constraints, LayoutContext, Rect, Size}
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
    fn measure(&mut self, children: &mut [&mut dyn RenderBox], constraints: Constraints, ctx: &mut dyn LayoutContext) -> Size {
        let loose = Constraints::loose();
        for child in children.iter_mut() {
            child.layout(loose, ctx);
        }
        let (total_children_width, max_height) = MarginAccumulator::horizontal_sum(&children.iter().map(|c| &**c).collect::<Vec<_>>());
        let total_width = total_children_width + self.spacing * (children.len().saturating_sub(1)) as f32;
        constraints.constrain(Size::new(total_width, max_height))
    }
}

impl LayoutArranger for HorizontalLayout {
    fn arrange(&mut self, children: &mut [&mut dyn RenderBox], inner_rect: Rect) -> Vec<Rect> {
        if children.is_empty() {
            return Vec::new();
        }
        let total_children_width: f32 = children.iter()
            .map(|c| c.size().width + c.margin().left + c.margin().right)
            .sum();
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
            let margin = child.margin();
            let child_size = child.size();
            let y = match self.cross_alignment {
                Alignment::Start => inner_rect.y + margin.top,
                Alignment::Center => inner_rect.y + (inner_rect.h - child_size.height) / 2.0,
                Alignment::End => inner_rect.y + inner_rect.h - child_size.height - margin.bottom,
                Alignment::Stretch => inner_rect.y + margin.top,
            };
            let height = if self.cross_alignment == Alignment::Stretch {
                (inner_rect.h - margin.top - margin.bottom).max(0.0)
            } else {
                child_size.height
            };
            let rect = Rect::new(
                current_x + margin.left,
                y,
                child_size.width,
                height,
            );
            rects.push(rect);
            current_x += child_size.width + margin.left + margin.right + self.spacing;
        }
        rects
    }
}


impl Default for HorizontalLayout {
    fn default() -> Self {
        Self::new()
    }
}