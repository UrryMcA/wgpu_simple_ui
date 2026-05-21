// src/widgets/container.rs
use super::widget::{Widget, MultiChildRenderObjectWidget};
use crate::common::render_box::{RenderBox, WidgetId};
use crate::common::{Primitives, types::*};
use crate::common::vertex::DrawCommand;
use crate::common::event::{Event, DragData};
use crate::common::layout_strategy::*;
use crate::layout::grid::GridLayout;
use crate::layout::horizontal::HorizontalLayout;
use crate::layout::vertical::VerticalLayout;
use crate::texture_manager::TextureManager;
use crate::ui::UiManager;

pub struct Container {
    children: Vec<Box<dyn Widget>>,
    axis: Axis,               // сохраняем для создания стратегии
    spacing: f32,
    alignment: Alignment,
    margin: EdgeInsets,
    padding: EdgeInsets,
    color: Option<UColor>,
    corner_radius: f32,
}



impl Container {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            axis: Axis::Vertical,
            spacing: 0.0,
            alignment: Alignment::Start,
            margin: EdgeInsets::default(),
            padding: EdgeInsets::default(),
            color: None,
            corner_radius: 0.0,
        }
    }

    pub fn vertical() -> Self { Self::new().axis(Axis::Vertical) }
    pub fn horizontal() -> Self { Self::new().axis(Axis::Horizontal) }
    pub fn grid(cols: usize, rows: usize, spacing_x: f32, spacing_y: f32) -> Self {
        Self::new().axis(Axis::Grid { cols, rows, spacing_x, spacing_y })
    }

    pub fn axis(mut self, axis: Axis) -> Self { self.axis = axis; self }
    pub fn spacing(mut self, spacing: f32) -> Self { self.spacing = spacing; self }
    pub fn alignment(mut self, alignment: Alignment) -> Self { self.alignment = alignment; self }
    pub fn margin(mut self, margin: EdgeInsets) -> Self { self.margin = margin; self }
    pub fn padding(mut self, padding: EdgeInsets) -> Self { self.padding = padding; self }
    pub fn color(mut self, color: UColor) -> Self { self.color = Some(color); self }
    pub fn corner_radius(mut self, radius: f32) -> Self { self.corner_radius = radius; self }
    pub fn add_child(mut self, child: Box<dyn Widget>) -> Self { self.children.push(child); self }
    pub fn add_children(mut self, children: Vec<Box<dyn Widget>>) -> Self { self.children.extend(children); self }
}

impl Widget for Container {
    fn min_size(&self) -> Size { Size::zero() }
    fn margin(&self) -> EdgeInsets { self.margin }
    fn padding(&self) -> EdgeInsets { self.padding }

    fn create_render_object(&self) -> Box<dyn RenderBox> {
        let children_render: Vec<Box<dyn RenderBox>> = self.children
            .iter()
            .map(|w| w.create_render_object())
            .collect();

        let strategy: Box<dyn LayoutStrategy> = match self.axis {
            Axis::Vertical => Box::new(VerticalLayout::new().with_spacing(self.spacing).with_cross_alignment(self.alignment)),
            Axis::Horizontal => Box::new(HorizontalLayout::new().with_spacing(self.spacing).with_cross_alignment(self.alignment)),
            Axis::Grid { cols, rows, spacing_x, spacing_y } => Box::new(GridLayout::new(cols, rows, spacing_x, spacing_y)),
        };

        Box::new(ContainerRenderObject {
            children: children_render,
            strategy,
            padding: self.padding,
            color: self.color,
            corner_radius: self.corner_radius,
            position: Point::default(),
            size: Size::default(),
            id: None,
        })
    }
}

impl MultiChildRenderObjectWidget for Container {
    fn children(&self) -> &[Box<dyn Widget>] {
        &self.children
    }
}


/// Объект рендеринга контейнера
pub struct ContainerRenderObject {
    children: Vec<Box<dyn RenderBox>>,
    strategy: Box<dyn LayoutStrategy>,
    padding: EdgeInsets,
    color: Option<UColor>,
    corner_radius: f32,
    position: Point,
    size: Size,
    id: Option<WidgetId>,
}

impl ContainerRenderObject {
    fn inner_rect(&self) -> Rect {
        Rect::new(
            self.position.x + self.padding.left,
            self.position.y + self.padding.top,
            self.size.width - self.padding.left - self.padding.right,
            self.size.height - self.padding.top - self.padding.bottom,
        )
    }
}

impl RenderBox for ContainerRenderObject {
    fn layout(&mut self, constraints: Constraints, ctx: &mut dyn LayoutContext) -> Size {
        // ========== ЭТАП 1: ИЗМЕРЕНИЕ ==========
        let mut children_refs: Vec<&mut dyn RenderBox> = self.children
            .iter_mut()
            .map(|c| c.as_mut())
            .collect();
        let inner_size = self.strategy.measure(&mut children_refs, constraints, ctx);
        // children_refs выходит из области видимости и заимствование заканчивается
        
        // ========== ВЫЧИСЛЕНИЕ СОБСТВЕННОГО РАЗМЕРА ==========
        let padded_size = self.padding.inflate(inner_size);
        let constrained_size = constraints.constrain(padded_size);
        self.size = constrained_size;
        
        // ========== ЭТАП 2: РАССТАНОВКА ==========
        let inner_rect = self.inner_rect(); // теперь можно immutable заимствовать self
        let mut children_refs2: Vec<&mut dyn RenderBox> = self.children
            .iter_mut()
            .map(|c| c.as_mut())
            .collect();
        let child_rects = self.strategy.arrange(&mut children_refs2, inner_rect);
        
        for (child, rect) in self.children.iter_mut().zip(child_rects) {
            child.set_position(Point::new(rect.x, rect.y));
        }
        
        self.size
    }

    fn set_position(&mut self, pos: Point) {
        self.position = pos;
    }

    fn position(&self) -> Point {
        self.position
    }

    fn size(&self) -> Size {
        self.size
    }

    fn render(
        &self,
        commands: &mut Vec<DrawCommand>,
        primitives: &dyn Primitives,
        textures: &TextureManager,
        ui_manager: &UiManager,
    ) {
        if let Some(color) = self.color {
            let rect = Rect::new(self.position.x, self.position.y, self.size.width, self.size.height);
            let bg = primitives.rounded_rect_vertices(rect, self.corner_radius, color);
            commands.push(DrawCommand { texture_id: 0, vertices: bg });
        }
        for child in &self.children {
            child.render(commands, primitives, textures, ui_manager);
        }
    }

    fn children(&self) -> &[Box<dyn RenderBox>] {
        &self.children
    }

    fn children_mut(&mut self) -> &mut [Box<dyn RenderBox>] {
        &mut self.children
    }

    fn hit_test(&self, point: Point) -> bool {
        let rect = Rect::new(self.position.x, self.position.y, self.size.width, self.size.height);
        rect.contains(point)
    }

    fn handle_event(&mut self, event: &Event, ui_manager: &mut UiManager) -> bool {
        for child in self.children_mut().iter_mut().rev() {
            if child.hit_test(event.point().unwrap_or(Point::default())) {
                if child.handle_event(event, ui_manager) {
                    return true;
                }
            }
        }
        false
    }

    fn can_focus(&self) -> bool {
        false
    }

    fn can_drop(&self, data: &DragData) -> bool {
        self.children.iter().any(|c| c.can_drop(data))
    }

    fn on_drag_enter(&mut self, data: &DragData, point: Point) {
        if let Some(child) = self.children_mut().iter_mut().find(|c| c.hit_test(point)) {
            child.on_drag_enter(data, point);
        }
    }

    fn on_drag_leave(&mut self) {
        for child in self.children_mut() {
            child.on_drag_leave();
        }
    }

    fn on_drop(&mut self, data: &DragData, point: Point) {
        if let Some(child) = self.children_mut().iter_mut().find(|c| c.hit_test(point) && c.can_drop(data)) {
            child.on_drop(data, point);
        }
    }

    fn widget_id(&self) -> Option<WidgetId> {
        self.id
    }
}

impl Drop for ContainerRenderObject {
    fn drop(&mut self) {}
}