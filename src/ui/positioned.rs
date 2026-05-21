use super::widget::{Widget, SingleChildRenderObjectWidget};
use crate::common::render_box::RenderBox;
use crate::common::types::*;
use crate::common::{DrawCommand, Primitives};
use crate::texture_manager::TextureManager;
use crate::ui::UiManager;

pub struct Positioned {
    child: Box<dyn Widget>,
    left: Option<f32>, top: Option<f32>, right: Option<f32>, bottom: Option<f32>,
    width: Option<f32>, height: Option<f32>,
}

impl Positioned {
    pub fn new(child: impl Widget + 'static) -> Self {
        Self {
            child: Box::new(child),
            left: None, top: None, right: None, bottom: None,
            width: None, height: None,
        }
    }
    pub fn left(mut self, v: f32) -> Self { self.left = Some(v); self }
    pub fn top(mut self, v: f32) -> Self { self.top = Some(v); self }
    pub fn right(mut self, v: f32) -> Self { self.right = Some(v); self }
    pub fn bottom(mut self, v: f32) -> Self { self.bottom = Some(v); self }
    pub fn width(mut self, v: f32) -> Self { self.width = Some(v); self }
    pub fn height(mut self, v: f32) -> Self { self.height = Some(v); self }
}

impl Widget for Positioned {
    fn min_size(&self) -> Size { self.child.min_size() }
    fn create_render_object(&self) -> Box<dyn RenderBox> {
        Box::new(PositionedRenderObject {
            child: self.child.create_render_object(),
            left: self.left, top: self.top, right: self.right, bottom: self.bottom,
            width: self.width, height: self.height,
            position: Point::default(),
            size: Size::default(),
        })
    }
}

impl SingleChildRenderObjectWidget for Positioned {
    fn child(&self) -> Option<&dyn Widget> { Some(self.child.as_ref()) }
}

struct PositionedRenderObject {
    child: Box<dyn RenderBox>,
    left: Option<f32>, top: Option<f32>, right: Option<f32>, bottom: Option<f32>,
    width: Option<f32>, height: Option<f32>,
    position: Point,
    size: Size,
}

impl RenderBox for PositionedRenderObject {
    fn layout(&mut self, constraints: Constraints, ctx: &mut dyn LayoutContext) -> Size {
        let child_constraints = Constraints::loose();
        let child_size = self.child.layout(child_constraints, ctx);
        let final_width = self.width.unwrap_or_else(|| {
            if let (Some(l), Some(r)) = (self.left, self.right) {
                (constraints.max_width - l - r).max(0.0)
            } else {
                child_size.width
            }
        });
        let final_height = self.height.unwrap_or_else(|| {
            if let (Some(t), Some(b)) = (self.top, self.bottom) {
                (constraints.max_height - t - b).max(0.0)
            } else {
                child_size.height
            }
        });
        let final_size = Size::new(final_width, final_height);
        self.size = final_size;
        final_size
    }
    fn set_position(&mut self, position: Point) {
        self.position = position;
        let child_x = position.x + self.left.unwrap_or(0.0);
        let child_y = position.y + self.top.unwrap_or(0.0);
        self.child.set_position(Point::new(child_x, child_y));
    }
    fn position(&self) -> Point { self.position }
    fn size(&self) -> Size { self.size }
    fn render(&self, commands: &mut Vec<DrawCommand>, primitives: &dyn Primitives, textures: &TextureManager, ui_manager: &UiManager) {
        self.child.render(commands, primitives, textures, ui_manager);
    }
}
