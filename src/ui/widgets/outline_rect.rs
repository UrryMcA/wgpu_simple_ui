// src/widgets/outline_rect.rs
use super::widget::{Widget, LeafRenderObjectWidget};
use crate::common::render_box::{RenderBox, WidgetId};
use crate::common::render_context::RenderContext;
use crate::common::{Primitives, Vertex, types::*};
use crate::common::event::{Event, KeyboardModifiers};
use crate::common::key::Key;
use crate::texture_manager::SamplerKind;
use crate::ui_manager::UiManager;

pub struct OutlineRect {
    id: Option<WidgetId>,
    width: f32,
    height: f32,
    corner_radius: f32,
    thickness: f32,
    color: UColor,
    margin: EdgeInsets,
}

impl OutlineRect {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            id: None,
            width,
            height,
            corner_radius: 8.0,
            thickness: 2.0,
            color: UColor([1.0, 1.0, 0.0, 1.0]), // жёлтый
            margin: EdgeInsets::default(),
        }
    }

    pub fn corner_radius(mut self, r: f32) -> Self {
        self.corner_radius = r;
        self
    }

    pub fn thickness(mut self, t: f32) -> Self {
        self.thickness = t;
        self
    }

    pub fn color(mut self, c: UColor) -> Self {
        self.color = c;
        self
    }

    pub fn margin(mut self, m: EdgeInsets) -> Self {
        self.margin = m;
        self
    }
    
    pub fn with_id(mut self, id: WidgetId) -> Self {
        self.id = Some(id);
        self
    }
}

impl Widget for OutlineRect {
    fn min_size(&self, _ctx: &mut dyn LayoutContext) -> Size {
        Size::new(self.width, self.height)
    }

    fn margin(&self) -> EdgeInsets {
        self.margin
    }

    fn padding(&self) -> EdgeInsets {
        EdgeInsets::default()
    }

    fn create_render_object(&mut self) -> Box<dyn RenderBox> {
        Box::new(OutlineRectRenderObject {
            id: self.id,
            width: self.width,
            height: self.height,
            radius: self.corner_radius,
            thickness: self.thickness,
            color: self.color,
            position: Point::default(),
            size: Size::default(),
            cached_vertices: Vec::new(),
            cached_indices: Vec::new(),
            dirty: true,
        })
    }

    fn set_id(&mut self, id: WidgetId) {
        self.id = Some(id);
    }

    fn id(&self) -> Option<WidgetId> {
        self.id
    }

}

impl LeafRenderObjectWidget for OutlineRect {}

struct OutlineRectRenderObject {
    id: Option<WidgetId>,
    width: f32,
    height: f32,
    radius: f32,
    thickness: f32,
    color: UColor,
    position: Point,
    size: Size,
    cached_vertices: Vec<Vertex>,
    cached_indices: Vec<u32>,
    dirty: bool,
}

impl OutlineRectRenderObject {
    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn rebuild_cache(&mut self, primitives: &dyn Primitives) {
        let rect = Rect::new(0.0, 0.0, self.size.width, self.size.height);
        let (verts, inds) = primitives.rounded_rect_outline_vertices_indices(
            rect,
            self.radius,
            self.thickness,
            self.color,
        );
        self.cached_vertices = verts;
        self.cached_indices = inds;
        self.dirty = false;
    }
}

impl RenderBox for OutlineRectRenderObject {
    fn layout(&mut self, constraints: Constraints, _ctx: &mut dyn LayoutContext) -> Size {
        let desired = Size::new(self.width, self.height);
        let new_size = constraints.constrain(desired);
        
        if (new_size.width - self.size.width).abs() > 0.01 
            || (new_size.height - self.size.height).abs() > 0.01 {
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

    fn position(&self) -> Point {
        self.position
    }

    fn size(&self) -> Size {
        self.size
    }

    fn render(&mut self, ctx: &mut RenderContext) {
        if self.dirty {
            self.rebuild_cache(ctx.primitives);
        }

        // Сдвигаем вершины в мировые координаты
        let mut world_verts = self.cached_vertices.clone();
        for v in &mut world_verts {
            v.position[0] += self.position.x;
            v.position[1] += self.position.y;
        }

        ctx.add_command(0, SamplerKind::Clamp, world_verts, self.cached_indices.clone());
    }

    fn children(&self) -> &[Box<dyn RenderBox>] {
        &[]
    }

    fn children_mut(&mut self) -> &mut [Box<dyn RenderBox>] {
        &mut []
    }

    fn hit_test(&self, point: Point) -> bool {
        Rect::new(self.position.x, self.position.y, self.size.width, self.size.height)
            .contains(point)
    }

    fn handle_event(&mut self, _event: &Event, _ui: &mut UiManager) -> bool {
        false
    }

    fn can_focus(&self) -> bool {
        false
    }

    fn set_focused(&mut self, _focused: bool) {}

    fn is_focused(&self) -> bool {
        false
    }

    fn handle_key_down(&mut self, _key: Key, _mod: KeyboardModifiers) -> bool {
        false
    }

    fn handle_char_input(&mut self, _ch: char) -> bool {
        false
    }

    fn margin(&self) -> EdgeInsets {
        EdgeInsets::default()
    }
    
    fn widget_id(&self) -> Option<WidgetId> {
        self.id
    }
    fn set_widget_id(&mut self, id: WidgetId) {
        self.id = Some(id);
    }

}