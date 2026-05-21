pub struct PaddingLayout {
    pub left: f32, pub right: f32, pub top: f32, pub bottom: f32,
}

impl LayoutStrategy for PaddingLayout {
    fn layout(&mut self, children: &mut [&mut dyn RenderBox], constraints: Constraints, ctx: &mut dyn LayoutContext) -> Vec<Rect> {
        assert_eq!(children.len(), 1);
        let inner_constraints = Constraints {
            min_width: constraints.min_width - self.left - self.right,
            max_width: constraints.max_width - self.left - self.right,
            min_height: constraints.min_height - self.top - self.bottom,
            max_height: constraints.max_height - self.top - self.bottom,
        }.loosen(); // clamp to non-negative
        let child_size = children[0].layout(inner_constraints, ctx);
        let rect = Rect::new(Point::new(self.left, self.top), child_size);
        vec![rect]
    }
}
