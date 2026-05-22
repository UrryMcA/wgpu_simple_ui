// src/widgets/stack.rs
use super::widget::{Widget, MultiChildRenderObjectWidget};
use crate::common::render_box::RenderBox;
use crate::common::render_context::RenderContext;
use crate::common::types::*;
use crate::common::event::Event;
use crate::ui_manager::UiManager;

pub struct Stack {
    children: Vec<Box<dyn Widget>>,
}

impl Stack {
    pub fn new(children: Vec<Box<dyn Widget>>) -> Self { Self { children } }
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
    fn create_render_object(&mut self) -> Box<dyn RenderBox> {
        let mut render_objects = Vec::new();
        for child in &mut self.children {
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
    fn layout(&mut self, constraints: Constraints, ctx: &mut dyn LayoutContext) -> Size {
        let mut max_w: f32 = 0.0;
        let mut max_h: f32 = 0.0;
        for child in &mut self.children {
            let size = child.layout(constraints, ctx);
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

    fn render(&mut self, ctx: &mut RenderContext) {
        for child in &mut self.children {
            child.render(ctx);
        }
    }

    fn children(&self) -> &[Box<dyn RenderBox>] { &self.children }
    fn children_mut(&mut self) -> &mut [Box<dyn RenderBox>] { &mut self.children }

    fn hit_test(&self, point: Point) -> bool {
        Rect::new(self.position.x, self.position.y, self.size.width, self.size.height).contains(point)
    }

    fn handle_event(&mut self, event: &Event, ui_manager: &mut UiManager) -> bool {
        for child in self.children_mut().iter_mut().rev() {
            if let Some(point) = event.point() {
                if child.hit_test(point) && child.handle_event(event, ui_manager) {
                    return true;
                }
            } else {
                // события без точки (например, клавиатурные) пробуем отдать детям, если они могут их обработать
                if child.handle_event(event, ui_manager) {
                    return true;
                }
            }
        }
        false
    }
}