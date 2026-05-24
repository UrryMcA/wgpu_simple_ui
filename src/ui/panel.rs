// src/widgets/panel.rs
use super::widget::Widget;
use crate::common::render_box::RenderBox;
use crate::common::render_context::RenderContext;
use crate::common::primitives::Primitives;
use crate::common::{Vertex, types::*};
use crate::common::event::{Event, KeyboardModifiers, DragData};
use crate::common::key::Key;
use crate::ui_manager::UiManager;

pub struct Panel {
    child: Option<Box<dyn Widget>>,
    background_color: UColor,
    border_color: UColor,
    border_thickness: f32,
    corner_radius: f32,
    margin: EdgeInsets,
    padding: EdgeInsets,
}

impl Panel {
    pub fn new() -> Self {
        Self {
            child: None,
            background_color: UColor([0.2, 0.2, 0.2, 1.0]),
            border_color: UColor([0.0, 0.0, 0.0, 0.0]),
            border_thickness: 0.0,
            corner_radius: 8.0,
            margin: EdgeInsets::default(),
            padding: EdgeInsets::all(12.0),
        }
    }

    pub fn content(mut self, child: Box<dyn Widget>) -> Self { self.child = Some(child); self }
    pub fn color(mut self, c: UColor) -> Self { self.background_color = c; self }
    pub fn background(mut self, c: UColor) -> Self { self.background_color = c; self }
    pub fn border_color(mut self, c: UColor) -> Self { self.border_color = c; self }
    pub fn border_thickness(mut self, t: f32) -> Self { self.border_thickness = t; self }
    pub fn corner_radius(mut self, r: f32) -> Self { self.corner_radius = r; self }
    pub fn margin(mut self, m: EdgeInsets) -> Self { self.margin = m; self }
    pub fn padding(mut self, p: EdgeInsets) -> Self { self.padding = p; self }
}

impl Widget for Panel {
    fn min_size(&self, ctx: &mut dyn LayoutContext) -> Size {
        let child = self.child.as_ref().expect("Panel requires a child widget");
        let child_min = child.min_size(ctx);
        let padded = self.padding.inflate(child_min);
        self.margin.inflate(padded)
    }
    fn margin(&self) -> EdgeInsets { self.margin }
    fn padding(&self) -> EdgeInsets { self.padding }
    
    fn create_render_object(&mut self) -> Box<dyn RenderBox> {
        let mut child = self.child.take().expect("Panel child must be set before render");
        Box::new(PanelRenderObject {
            child: child.create_render_object(),
            background_color: self.background_color,
            border_color: self.border_color,
            border_thickness: self.border_thickness,
            corner_radius: self.corner_radius,
            padding: self.padding,
            margin: self.margin,
            position: Point::default(),
            size: Size::default(),
            id: None,
            cached_bg_vertices: Vec::new(),
            cached_bg_indices: Vec::new(),
            cached_border_vertices: Vec::new(),
            cached_border_indices: Vec::new(),
            dirty: true,
        })
    }
}

struct PanelRenderObject {
    child: Box<dyn RenderBox>,
    background_color: UColor,
    border_color: UColor,
    border_thickness: f32,
    corner_radius: f32,
    padding: EdgeInsets,
    margin: EdgeInsets,
    position: Point,
    size: Size,
    id: Option<u64>,
    cached_bg_vertices: Vec<Vertex>,
    cached_bg_indices: Vec<u32>,
    cached_border_vertices: Vec<Vertex>,
    cached_border_indices: Vec<u32>,
    dirty: bool,
}

impl PanelRenderObject {
    fn mark_dirty(&mut self) { self.dirty = true; }

    fn rebuild_cache(&mut self, primitives: &dyn Primitives) {
        let rect = Rect::new(
            self.position.x + self.margin.left,
            self.position.y + self.margin.top,
            self.size.width - self.margin.left - self.margin.right,
            self.size.height - self.margin.top - self.margin.bottom,
        );

        // Фон
        if self.background_color.0[3] > 0.0 {
            let (v, i) = primitives.rounded_rect_vertices_indices(rect, self.corner_radius, self.background_color);
            self.cached_bg_vertices = v;
            self.cached_bg_indices = i;
        } else {
            self.cached_bg_vertices.clear();
            self.cached_bg_indices.clear();
        }

        // Граница
        if self.border_thickness > 0.0 && self.border_color.0[3] > 0.0 {
            let (v, i) = primitives.rounded_rect_outline_vertices_indices(rect, self.corner_radius, self.border_thickness, self.border_color);
            self.cached_border_vertices = v;
            self.cached_border_indices = i;
        } else {
            self.cached_border_vertices.clear();
            self.cached_border_indices.clear();
        }
        self.dirty = false;
    }
}

impl RenderBox for PanelRenderObject {
    fn layout(&mut self, constraints: Constraints, ctx: &mut dyn LayoutContext) -> Size {
        let total_h = self.margin.left + self.margin.right + self.padding.left + self.padding.right;
        let total_v = self.margin.top + self.margin.bottom + self.padding.top + self.padding.bottom;
        
        let inner = Constraints {
            min_width: (constraints.min_width - total_h).max(0.0),
            max_width: (constraints.max_width - total_h).max(0.0),
            min_height: (constraints.min_height - total_v).max(0.0),
            max_height: (constraints.max_height - total_v).max(0.0),
        };
        let child_size = self.child.layout(inner, ctx);
        let new_size = constraints.constrain(Size::new(child_size.width + total_h, child_size.height + total_v));
        
        if new_size != self.size {
            self.size = new_size;
            self.mark_dirty();
        }
        let inner_origin = Point::new(
            self.position.x + self.margin.left + self.padding.left,
            self.position.y + self.margin.top + self.padding.top,
        );
        self.child.set_position(inner_origin);
        new_size
    }

    fn set_position(&mut self, pos: Point) {
        if self.position != pos { self.position = pos; self.mark_dirty(); }
    }
    fn position(&self) -> Point { self.position }
    fn size(&self) -> Size { self.size }

    fn render(&mut self, ctx: &mut RenderContext) {
        if self.dirty { self.rebuild_cache(ctx.primitives); }
        
        if !self.cached_bg_vertices.is_empty() {
            ctx.add_command(0, self.cached_bg_vertices.clone(), self.cached_bg_indices.clone());
        }
        if !self.cached_border_vertices.is_empty() {
            ctx.add_command(0, self.cached_border_vertices.clone(), self.cached_border_indices.clone());
        }
        self.child.render(ctx);
    }

    fn children(&self) -> &[Box<dyn RenderBox>] { std::slice::from_ref(&self.child) }
    fn children_mut(&mut self) -> &mut [Box<dyn RenderBox>] { std::slice::from_mut(&mut self.child) }
    fn hit_test(&self, point: Point) -> bool {
        Rect::new(self.position.x + self.margin.left, self.position.y + self.margin.top,
                  self.size.width - self.margin.left - self.margin.right,
                  self.size.height - self.margin.top - self.margin.bottom).contains(point)
    }
    fn handle_event(&mut self, event: &Event, ui_manager: &mut UiManager) -> bool {
        if let Some(point) = event.point() { if !self.hit_test(point) { return false; } }
        self.child.handle_event(event, ui_manager)
    }
    fn can_focus(&self) -> bool { self.child.can_focus() }
    fn set_focused(&mut self, f: bool) { self.child.set_focused(f); }
    fn is_focused(&self) -> bool { self.child.is_focused() }
    fn handle_key_down(&mut self, k: Key, m: KeyboardModifiers) -> bool { self.child.handle_key_down(k, m) }
    fn handle_key_up(&mut self, k: Key, m: KeyboardModifiers) -> bool { self.child.handle_key_up(k, m) }
    fn handle_char_input(&mut self, c: char) -> bool { self.child.handle_char_input(c) }
    fn can_drag(&self) -> bool { self.child.can_drag() }
    fn drag_data(&self) -> Option<DragData> { self.child.drag_data() }
    fn on_drag_start(&mut self, p: Point) { self.child.on_drag_start(p); }
    fn on_drag_move(&mut self, p: Point) { self.child.on_drag_move(p); }
    fn on_drag_end(&mut self, c: bool) { self.child.on_drag_end(c); }
    fn can_drop(&self, d: &DragData) -> bool { self.child.can_drop(d) }
    fn on_drag_enter(&mut self, d: &DragData, p: Point) { self.child.on_drag_enter(d, p); }
    fn on_drag_leave(&mut self) { self.child.on_drag_leave(); }
    fn on_drop(&mut self, d: &DragData, p: Point) { self.child.on_drop(d, p); }
    fn widget_id(&self) -> Option<u64> { self.id }
}