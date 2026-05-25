// src/widgets/label.rs
use super::widget::{Widget, LeafRenderObjectWidget};
use crate::common::render_box::RenderBox;
use crate::common::render_context::RenderContext;
use crate::common::{Vertex, types::*};

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
            // Кэш
            cached_vertices: Vec::new(),
            cached_indices: Vec::new(),
            dirty: true,
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
    // Кэш
    cached_vertices: Vec<Vertex>,
    cached_indices: Vec<u32>,
    dirty: bool,
}

impl LabelRenderObject {
    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn rebuild_cache(&mut self, ctx: &mut RenderContext) {
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
            self.cached_vertices = verts;
            self.cached_indices = inds;
        } else {
            self.cached_vertices.clear();
            self.cached_indices.clear();
        }
        self.dirty = false;
    }
}

impl RenderBox for LabelRenderObject {
    fn layout(&mut self, constraints: Constraints, ctx: &mut dyn LayoutContext) -> Size {
        let new_size = ctx.measure_text_with_font(&self.font_name, &self.text, self.font_size, constraints.max_width);
        let new_size = constraints.constrain(new_size);
        if new_size != self.size {
            self.size = new_size;
            self.mark_dirty();
        }
        new_size
    }

    fn set_position(&mut self, pos: Point) {
        if self.position != pos {
            self.position = pos;
            self.mark_dirty();
        }
    }

    fn position(&self) -> Point { self.position }
    fn size(&self) -> Size { self.size }

    fn render(&mut self, ctx: &mut RenderContext) {
        if self.dirty {
            self.rebuild_cache(ctx);
        }
        if !self.cached_vertices.is_empty() {
            ctx.add_command(
                ctx.font_system.get_font(&self.font_name).map(|f| f.texture_id()).unwrap_or(0),
                self.cached_vertices.clone(),
                self.cached_indices.clone(),
            );
        }
    }

    fn children(&self) -> &[Box<dyn RenderBox>] { &[] }
    fn children_mut(&mut self) -> &mut [Box<dyn RenderBox>] { &mut [] }
    fn hit_test(&self, _point: Point) -> bool { false }
    fn widget_id(&self) -> Option<u64> { None }
    fn margin(&self) -> EdgeInsets { self.margin }
}