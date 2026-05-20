use super::widget::{Widget, SingleChildRenderObjectWidget};
use crate::common::render_box::RenderBox;
use crate::common::types::*;
use crate::common::{DrawCommand, Primitives};
use crate::texture_manager::TextureManager;
use crate::ui::UiManager;

pub struct Container {
    child: Option<Box<dyn Widget>>,
    margin: EdgeInsets,
    padding: EdgeInsets,
    color: Option<UColor>,
    corner_radius: f32,
}

impl Container {
    pub fn new(child: impl Widget + 'static) -> Self {
        Self {
            child: Some(Box::new(child)),
            margin: EdgeInsets::default(),
            padding: EdgeInsets::default(),
            color: None,
            corner_radius: 0.0,
        }
    }
    pub fn margin(mut self, m: EdgeInsets) -> Self { self.margin = m; self }
    pub fn padding(mut self, p: EdgeInsets) -> Self { self.padding = p; self }
    pub fn color(mut self, c: UColor) -> Self { self.color = Some(c); self }
    pub fn corner_radius(mut self, r: f32) -> Self { self.corner_radius = r; self }
}

impl Widget for Container {
    fn min_size(&self) -> Size {
        let child_min = self.child.as_ref().map(|c| c.min_size()).unwrap_or_default();
        self.padding.inflate(child_min)
    }
    fn margin(&self) -> EdgeInsets { self.margin }
    fn padding(&self) -> EdgeInsets { self.padding }
    fn create_render_object(&self) -> Box<dyn RenderBox> {
        let child_render = self.child.as_ref().map(|c| c.create_render_object());
        Box::new(ContainerRenderObject {
            child_render,
            margin: self.margin,
            padding: self.padding,
            color: self.color,
            radius: self.corner_radius,
            position: Point::default(),
            size: Size::default(),
        })
    }
}

impl SingleChildRenderObjectWidget for Container {
    fn child(&self) -> Option<&dyn Widget> {
        self.child.as_ref().map(|c| c.as_ref())
    }
}

struct ContainerRenderObject {
    child_render: Option<Box<dyn RenderBox>>,
    margin: EdgeInsets,
    padding: EdgeInsets,
    color: Option<UColor>,
    radius: f32,
    position: Point,
    size: Size,
}

impl RenderBox for ContainerRenderObject {
    fn layout(&mut self, constraints: Constraints) -> Size {
        let inner_constraints = Constraints {
            min_width: (constraints.min_width - self.margin.left - self.margin.right).max(0.0),
            max_width: (constraints.max_width - self.margin.left - self.margin.right).max(0.0),
            min_height: (constraints.min_height - self.margin.top - self.margin.bottom).max(0.0),
            max_height: (constraints.max_height - self.margin.top - self.margin.bottom).max(0.0),
        };
        let child_size = if let Some(child) = &mut self.child_render {
            let child_constraints = Constraints {
                min_width: (inner_constraints.min_width - self.padding.left - self.padding.right).max(0.0),
                max_width: (inner_constraints.max_width - self.padding.left - self.padding.right).max(0.0),
                min_height: (inner_constraints.min_height - self.padding.top - self.padding.bottom).max(0.0),
                max_height: (inner_constraints.max_height - self.padding.top - self.padding.bottom).max(0.0),
            };
            child.layout(child_constraints)
        } else {
            Size::default()
        };
        let total_size = self.padding.inflate(child_size);
        let final_size = total_size.inflate(self.margin);
        self.size = final_size;
        final_size
    }
    fn set_position(&mut self, position: Point) {
        self.position = position;
        let child_pos = Point {
            x: position.x + self.margin.left + self.padding.left,
            y: position.y + self.margin.top + self.padding.top,
        };
        if let Some(child) = &mut self.child_render {
            child.set_position(child_pos);
        }
    }
    fn position(&self) -> Point { self.position }
    fn size(&self) -> Size { self.size }
    fn render(&self, commands: &mut Vec<DrawCommand>, primitives: &dyn Primitives, textures: &TextureManager, ui_manager: &UiManager) {
        if let Some(color) = self.color {
            let rect = Rect::new(self.position.x, self.position.y, self.size.width, self.size.height);
            let bg = primitives.rounded_rect_vertices(rect, self.radius, color);
            commands.push(crate::common::DrawCommand { texture_id: 0, vertices: bg });
        }
        if let Some(child) = &self.child_render {
            child.render(commands, primitives, textures, ui_manager);
        }
    }
}