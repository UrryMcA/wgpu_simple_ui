use crate::common::{layout_strategy::LayoutStrategy, render_box::RenderBox, types::*};

pub struct BorderLayout {
    north_size: Option<f32>,
    south_size: Option<f32>,
    west_size: Option<f32>,
    east_size: Option<f32>,
    h_gap: f32,
    v_gap: f32,
}

impl LayoutStrategy for BorderLayout {
    fn layout(&mut self, children: &mut [&mut dyn RenderBox], constraints: Constraints, ctx: &mut dyn LayoutContext) -> Vec<Rect> {
        // children должны быть в порядке: [North, South, West, East, Center] (или по именам)
        let mut rects = vec![Rect::default(); children.len()];
        let mut remaining = Rect::from_size(constraints.constrain(Size::new(f32::INFINITY, f32::INFINITY)));
        
        // Регион North
        if let Some(child) = children.get_mut(0) {
            let size = child.layout(Constraints::tight(remaining.size()), ctx);
            rects[0] = Rect::new(remaining.origin, size);
            remaining.origin.y += size.height + self.v_gap;
            remaining.size.height -= size.height + self.v_gap;
        }
        // аналогично South, West, East...
        // Центр получает оставшееся пространство
        // ...
        rects
    }
}
