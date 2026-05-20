use super::widget::{Widget, MultiChildRenderObjectWidget};
use super::render_box::RenderBox;
use super::types::{Size, Point, Constraints, EdgeInsets};
use crate::common::{DrawCommand, Primitives};
use crate::texture_manager::TextureManager;
use crate::ui::UiManager;

pub struct Stack {
    children: Vec<Box<dyn Widget>>,
}

impl Stack {
    pub fn new(children: Vec<Box<dyn Widget>>) -> Self {
        Self { children }
    }
}

impl Widget for Stack {
    fn min_size(&self) -> Size {
        let mut max_w: f32 = 0.0;
        let mut max_h: f32 = 0.0;
        for child in &self.children {
            let s = child.min_size();
            max_w = max_w.max(s.width);
            max_h = max_h.max(s.height);
        }
        Size::new(max_w, max_h)
    }
    fn create_render_object(&self) -> Box<dyn RenderBox> {
        let mut render_objects = Vec::new();
        for child in &self.children {
            render_objects.push(child.create_render_object());
        }
        Box::new(StackRenderObject {
            children: render_objects,
            position: Point::default(),
            size: Size::default(),
        })
    }
}

impl MultiChildRenderObjectWidget for Stack {
    fn children(&self) -> &[Box<dyn Widget>] { &self.children }
}

struct StackRenderObject {
    children: Vec<Box<dyn RenderBox>>,
    position: Point,
    size: Size,
}

impl RenderBox for StackRenderObject {
    fn layout(&mut self, constraints: Constraints) -> Size {
        let mut max_w: f32 = 0.0;
        let mut max_h: f32 = 0.0;
        for child in &mut self.children {
            let size = child.layout(constraints);
            max_w = max_w.max(size.width);
            max_h = max_h.max(size.height);
        }
        let final_size = Size::new(max_w, max_h);
        self.size = final_size;
        final_size
    }
    fn set_position(&mut self, position: Point) {
        self.position = position;
        for child in &mut self.children {
            child.set_position(position);
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
