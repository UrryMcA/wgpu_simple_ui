use crate::common::{
    layout_strategy::{LayoutArranger},
    render_box::RenderBox,
    types::{Alignment, Rect},
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

impl LayoutArranger for VerticalLayout {
    fn arrange(&mut self, children: &mut [&mut dyn RenderBox], inner_rect: Rect) -> Vec<Rect> {
        if children.is_empty() {
            return Vec::new();
        }
        let total_children_height: f32 = children.iter()
            .map(|c| c.size().height + c.margin().top + c.margin().bottom)
            .sum();
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
            let margin = child.margin();
            let child_size = child.size();
            let x = match self.cross_alignment {
                Alignment::Start => inner_rect.x + margin.left,
                Alignment::Center => inner_rect.x + (inner_rect.w - child_size.width) / 2.0,
                Alignment::End => inner_rect.x + inner_rect.w - child_size.width - margin.right,
                Alignment::Stretch => inner_rect.x + margin.left,
            };
            let width = if self.cross_alignment == Alignment::Stretch {
                (inner_rect.w - margin.left - margin.right).max(0.0)
            } else {
                child_size.width
            };
            let rect = Rect::new(
                x,
                current_y + margin.top,
                width,
                child_size.height,
            );
            rects.push(rect);
            current_y += child_size.height + margin.top + margin.bottom + self.spacing;
        }
        rects
    }
}

impl Default for VerticalLayout {
    fn default() -> Self {
        Self::new()
    }
}