// src/widgets/image.rs
use super::widget::{Widget, LeafRenderObjectWidget};
use crate::common::render_box::RenderBox;
use crate::common::render_context::RenderContext;
use crate::common::{Primitives, Vertex, types::*};
use crate::texture_manager::SamplerKind;

pub struct Image {
    texture_id: u64,
    size: Size,
    margin: EdgeInsets,
}

impl Image {
    pub fn new(texture_id: u64, width: f32, height: f32) -> Self {
        Self {
            texture_id,
            size: Size::new(width, height),
            margin: EdgeInsets::default(),
        }
    }
    pub fn margin(mut self, m: EdgeInsets) -> Self { self.margin = m; self }
}

impl Widget for Image {
    fn min_size(&self, _ctx: &mut dyn LayoutContext) -> Size { self.size }
    fn margin(&self) -> EdgeInsets { self.margin }
    fn create_render_object(&mut self) -> Box<dyn RenderBox> {
        Box::new(ImageRenderObject {
            texture_id: self.texture_id,
            size: self.size,
            margin: self.margin,
            position: Point::default(),
            cached_vertices: Vec::new(),
            cached_indices: Vec::new(),
            dirty: true,
        })
    }
}

impl LeafRenderObjectWidget for Image {}

struct ImageRenderObject {
    texture_id: u64,
    size: Size,
    margin: EdgeInsets,
    position: Point,
    // Кэш
    cached_vertices: Vec<Vertex>,
    cached_indices: Vec<u32>,
    dirty: bool,
}

impl ImageRenderObject {
    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn rebuild_cache(&mut self, primitives: &dyn Primitives) {
        let rect = Rect::new(self.position.x, self.position.y, self.size.width, self.size.height);
        let tex_coords = TexCoords::new(0.0, 0.0, 1.0, 1.0);
        let color = UColor([1.0, 1.0, 1.0, 1.0]);
        let (verts, inds) = primitives.textured_rect_vertices_indices(rect, tex_coords, color);
        self.cached_vertices = verts;
        self.cached_indices = inds;
        self.dirty = false;
    }
}

impl RenderBox for ImageRenderObject {
    fn layout(&mut self, constraints: Constraints, _ctx: &mut dyn LayoutContext) -> Size {
        let new_size = constraints.constrain(self.size);
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
            self.rebuild_cache(ctx.primitives);
        }
        ctx.add_command(self.texture_id, SamplerKind::Clamp, self.cached_vertices.clone(), self.cached_indices.clone());
    }

    fn children(&self) -> &[Box<dyn RenderBox>] { &[] }
    fn children_mut(&mut self) -> &mut [Box<dyn RenderBox>] { &mut [] }
    fn hit_test(&self, point: Point) -> bool {
        Rect::new(self.position.x, self.position.y, self.size.width, self.size.height).contains(point)
    }
    fn margin(&self) -> EdgeInsets { self.margin }
}