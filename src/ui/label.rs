// src/widgets/label.rs
use super::widget::{Widget, LeafRenderObjectWidget};
use crate::common::render_box::RenderBox;
use crate::common::render_context::RenderContext;
use crate::common::types::*;

pub struct Label {
    text: String,
    font_name: String,
    font_size: f32,
    color: UColor,
    margin: EdgeInsets,
}

impl Label {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            font_name: "default".into(),
            font_size: 16.0,
            color: UColor([1.0, 1.0, 1.0, 1.0]),
            margin: EdgeInsets::default(),
        }
    }
    pub fn font_size(mut self, size: f32) -> Self { self.font_size = size; self }
    pub fn color(mut self, c: UColor) -> Self { self.color = c; self }
    pub fn margin(mut self, m: EdgeInsets) -> Self { self.margin = m; self }
}

impl Widget for Label {
    fn min_size(&self, ctx: &mut dyn LayoutContext) -> Size {
        ctx.measure_text_with_font(&self.font_name, &self.text, self.font_size, f32::INFINITY)
    }
    fn margin(&self) -> EdgeInsets { self.margin }
    fn create_render_object(&mut self) -> Box<dyn RenderBox> {
        Box::new(LabelRenderObject {
            text: self.text.clone(),
            font_name: self.font_name.clone(),
            font_size: self.font_size,
            color: self.color,
            margin: self.margin,
            position: Point::default(),
            size: Size::default(),
        })
    }
}

impl LeafRenderObjectWidget for Label {}

struct LabelRenderObject {
    text: String,
    font_name: String,
    font_size: f32,
    color: UColor,
    margin: EdgeInsets,
    position: Point,
    size: Size,
}

impl RenderBox for LabelRenderObject {
    fn layout(&mut self, constraints: Constraints, ctx: &mut dyn LayoutContext) -> Size {
        let text_size = ctx.measure_text_with_font(&self.font_name, &self.text, self.font_size, constraints.max_width);
        let constrained = constraints.constrain(text_size);
        self.size = constrained;
        constrained
    }

    fn set_position(&mut self, pos: Point) { self.position = pos; }
    fn position(&self) -> Point { self.position }
    fn size(&self) -> Size { self.size }

    fn render(&mut self, ctx: &mut RenderContext) {
        if let Some(font) = ctx.font_system.get_font(&self.font_name) {
            let (verts, inds) = ctx.font_system.generate_text_vertices_with_font(
                font,
                &self.text,
                self.position.x,
                self.position.y,
                self.font_size,
                self.color,
                ctx.primitives,
            );
            ctx.add_command(font.texture_id(), verts, inds);
        }
    }

    fn children(&self) -> &[Box<dyn RenderBox>] { &[] }
    fn children_mut(&mut self) -> &mut [Box<dyn RenderBox>] { &mut [] }
    fn hit_test(&self, _point: Point) -> bool { false }
    fn widget_id(&self) -> Option<u64> { None }
    fn margin(&self) -> EdgeInsets { self.margin }
}