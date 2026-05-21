use crate::common::{layout_strategy::LayoutStrategy, render_box::RenderBox, types::{Constraints, LayoutContext, Rect}};


#[derive(Clone)]
pub struct HorizontalLayout {
    pub spacing: f32,
    pub alignment: MainAxisAlignment, // Start, Center, End, SpaceBetween, SpaceAround
}

impl HorizontalLayout {
    pub fn new() -> Self {
        Self { spacing: 0.0, alignment: MainAxisAlignment::Start }
    }
    
    pub fn with_spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }
}

impl LayoutStrategy for HorizontalLayout {
    fn layout(
        &mut self,
        children: &mut [&mut dyn RenderBox],
        constraints: Constraints,
        ctx: &mut dyn LayoutContext,
    ) -> Vec<Rect> {
        if children.is_empty() { return vec![]; }
        
        // 1. Сначала запрашиваем размер каждого ребёнка с ограничением по высоте = max_height,
        //    а по ширине – flexible (но можно и жестко, если надо).
        let mut child_sizes = Vec::with_capacity(children.len());
        for child in children.iter_mut() {
            let child_constraints = Constraints {
                min_width: 0.0,
                max_width: constraints.max_width, // будет уточнено позже, если нужно деление поровну
                min_height: constraints.min_height,
                max_height: constraints.max_height,
            };
            let size = child.layout(child_constraints, ctx);
            child_sizes.push(size);
        }
        
        // 2. Вычисляем суммарную ширину всех детей + отступы.
        let total_children_width: f32 = child_sizes.iter().map(|s| s.width).sum();
        let total_spacing = self.spacing * (children.len() - 1) as f32;
        let needed_width = total_children_width + total_spacing;
        
        // 3. Определяем начальный сдвиг по X в зависимости от alignment и доступной ширины.
        let available_width = constraints.max_width.min(constraints.min_width.max(needed_width));
        let start_x = match self.alignment {
            MainAxisAlignment::Start => 0.0,
            MainAxisAlignment::Center => (available_width - needed_width) / 2.0,
            MainAxisAlignment::End => available_width - needed_width,
            MainAxisAlignment::SpaceBetween => 0.0, // handled later
            MainAxisAlignment::SpaceAround => 0.0,
        };
        
        let mut rects = Vec::with_capacity(children.len());
        let mut current_x = start_x;
        
        for (i, size) in child_sizes.iter().enumerate() {
            // SpaceBetween/SpaceAround: пересчитываем интервалы
            let x = current_x;
            let y = 0.0; // контейнер сам решит вертикальное выравнивание (или передать в layout)
            let rect = Rect::new(Point::new(x, y), *size);
            rects.push(rect);
            current_x += size.width + self.spacing;
        }
        
        rects
    }
}
