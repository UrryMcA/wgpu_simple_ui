use super::widget::{Widget, MultiChildRenderObjectWidget};
use crate::common::render_box::RenderBox;
use crate::common::types::*;
use crate::common::{DrawCommand, Primitives};
use crate::texture_manager::TextureManager;
use crate::ui::UiManager;

pub struct Row {
    children: Vec<Box<dyn Widget>>,
    spacing: f32,
}

impl Row {
    pub fn new(children: Vec<Box<dyn Widget>>, spacing: f32) -> Self {
        Self { children, spacing }
    }
}

impl Widget for Row {
    fn min_size(&self) -> Size {
        let mut total_width = 0.0;
        let mut max_height: f32 = 0.0;
        for child in &self.children {
            let s = child.min_size();
            total_width += s.width;
            if total_width > 0.0 { total_width += self.spacing; }
            max_height = max_height.max(s.height);
        }
        if total_width > 0.0 { total_width -= self.spacing; }
        Size::new(total_width, max_height)
    }
    fn create_render_object(&self) -> Box<dyn RenderBox> {
        let mut render_objects = Vec::new();
        for child in &self.children {
            render_objects.push(child.create_render_object());
        }
        Box::new(RowRenderObject {
            children: render_objects,
            spacing: self.spacing,
            position: Point::default(),
            size: Size::default(),
        })
    }
}

impl MultiChildRenderObjectWidget for Row {
    fn children(&self) -> &[Box<dyn Widget>] { &self.children }
}

struct RowRenderObject {
    children: Vec<Box<dyn RenderBox>>,
    spacing: f32,
    position: Point,
    size: Size,
}

impl RenderBox for RowRenderObject {
    fn layout(&mut self, constraints: Constraints) -> Size {
        let mut child_sizes = Vec::new();
        let mut total_width = 0.0;
        let mut max_height: f32 = 0.0;
        let children_len = self.children.len();
        for child in &mut self.children {
            let child_constraints = Constraints {
                min_width: 0.0,
                max_width: constraints.max_width - total_width - self.spacing * (children_len - 1) as f32,
                min_height: constraints.min_height,
                max_height: constraints.max_height,
            };
            let size = child.layout(child_constraints);
            child_sizes.push(size);
            total_width += size.width;
            max_height = max_height.max(size.height);
            total_width += self.spacing;
        }
        if total_width > 0.0 { total_width -= self.spacing; }
        let final_size = Size::new(total_width, max_height);
        self.size = final_size;
        final_size
    }
    fn set_position(&mut self, position: Point) {
        self.position = position;
        let mut x = position.x;
        for child in &mut self.children {
            child.set_position(Point::new(x, position.y));
            x += child.size().width + self.spacing;
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