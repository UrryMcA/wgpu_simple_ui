use super::widget::{Widget, MultiChildRenderObjectWidget};
use crate::common::render_box::RenderBox;
use crate::common::types::{Constraints, LayoutContext, Point, Size};
use crate::common::{DrawCommand, Primitives};
use crate::texture_manager::TextureManager;
use crate::ui::UiManager;

pub struct Column {
    children: Vec<Box<dyn Widget>>,
    spacing: f32,
}

impl Column {
    pub fn new(children: Vec<Box<dyn Widget>>, spacing: f32) -> Self {
        Self { children, spacing }
    }
}

impl Widget for Column {
    fn min_size(&self) -> Size {
        let mut total_height = 0.0;
        let mut max_width: f32 = 0.0;
        for child in &self.children {
            let s = child.min_size();
            total_height += s.height;
            if total_height > 0.0 { total_height += self.spacing; }
            max_width = max_width.max(s.width);
        }
        if total_height > 0.0 { total_height -= self.spacing; }
        Size::new(max_width, total_height)
    }
    fn create_render_object(&self) -> Box<dyn RenderBox> {
        let mut render_objects = Vec::new();
        for child in &self.children {
            render_objects.push(child.create_render_object());
        }
        Box::new(ColumnRenderObject {
            children: render_objects,
            spacing: self.spacing,
            position: Point::default(),
            size: Size::default(),
        })
    }
}

impl MultiChildRenderObjectWidget for Column {
    fn children(&self) -> &[Box<dyn Widget>] { &self.children }
}

struct ColumnRenderObject {
    children: Vec<Box<dyn RenderBox>>,
    spacing: f32,
    position: Point,
    size: Size,
}

impl RenderBox for ColumnRenderObject {
    fn layout(&mut self, constraints: Constraints, ctx: &mut dyn LayoutContext) -> Size {
        let mut child_sizes = Vec::new();
        let mut total_height = 0.0;
        let mut max_width: f32 = 0.0;
        let children_len = self.children.len();
        for child in &mut self.children {
            let child_constraints = Constraints {
                min_width: constraints.min_width,
                max_width: constraints.max_width,
                min_height: 0.0,
                max_height: constraints.max_height - total_height - self.spacing * (children_len - 1) as f32,
            };
            let size = child.layout(child_constraints, ctx);
            child_sizes.push(size);
            total_height += size.height;
            max_width = max_width.max(size.width);
            total_height += self.spacing;
        }
        if total_height > 0.0 { total_height -= self.spacing; }
        let final_size = Size::new(max_width, total_height);
        self.size = final_size;
        final_size
    }
    fn set_position(&mut self, position: Point) {
        self.position = position;
        let mut y = position.y;
        for child in &mut self.children {
            child.set_position(Point::new(position.x, y));
            y += child.size().height + self.spacing;
        }
    }
    fn position(&self) -> Point { self.position }
    fn size(&self) -> Size { self.size }
    fn render(&self, commands: &mut Vec<DrawCommand>, primitives: &dyn Primitives, textures: &TextureManager, ui_manager: &UiManager) {
        for child in &self.children {
            child.render(commands, primitives, textures, ui_manager);
        }
    }
}