// layout/vertical.rs
impl LayoutStrategy for VerticalLayout {
    fn desired_size(&self, children: &[Box<dyn RenderBox>]) -> Size {
        let total_height: f32 = children.iter().map(|c| c.size().height).sum::<f32>()
            + self.spacing * (children.len() - 1) as f32;
        let max_width = children.iter().map(|c| c.size().width).fold(0.0, f32::max);
        Size::new(max_width, total_height)
    }

    fn arrange(&mut self, children: &mut [&mut dyn RenderBox], inner_rect: Rect) -> Vec<Rect> {
        let mut rects = Vec::with_capacity(children.len());
        let mut current_y = inner_rect.origin.y;
        for child in children {
            let child_size = child.size();
            let x = match self.alignment {
                MainAxisAlignment::Start => inner_rect.origin.x,
                MainAxisAlignment::Center => inner_rect.origin.x + (inner_rect.size.width - child_size.width) / 2.0,
                MainAxisAlignment::End => inner_rect.origin.x + inner_rect.size.width - child_size.width,
                // SpaceBetween, SpaceAround требуют знания всех размеров, можно реализовать отдельно
                _ => inner_rect.origin.x,
            };
            let rect = Rect::new(Point::new(x, current_y), child_size);
            rects.push(rect);
            current_y += child_size.height + self.spacing;
        }
        // Если alignment = SpaceBetween, нужно пересчитать, растянув промежутки
        // Для краткости опускаем, но логику можно добавить.
        rects
    }
}
