// src/widgets/button.rs
use super::widget::{Widget, LeafRenderObjectWidget};
use crate::common::render_box::{RenderBox, WidgetId};
use crate::common::render_context::RenderContext;
use crate::common::{Primitives, Vertex, types::*};
use crate::common::event::{Event, KeyboardModifiers, DragData};
use crate::common::key::Key;
use crate::ui_manager::UiManager;

pub struct Button {
    text: String,
    font_name: String,
    padding: EdgeInsets,
    margin: EdgeInsets,
    color: UColor,
    border_color: UColor,
    border_thickness: f32,
    corner_radius: f32,
    on_click: Option<Box<dyn FnMut() + Send>>,
}

impl Button {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            font_name: "default".into(),
            padding: EdgeInsets::all(8.0),
            margin: EdgeInsets::default(),
            color: UColor([0.2, 0.3, 0.5, 1.0]),
            border_color: UColor([0.0, 0.0, 0.0, 0.0]),
            border_thickness: 0.0,
            corner_radius: 4.0,
            on_click: None,
        }
    }
    pub fn padding(mut self, p: EdgeInsets) -> Self { self.padding = p; self }
    pub fn margin(mut self, m: EdgeInsets) -> Self { self.margin = m; self }
    pub fn color(mut self, c: UColor) -> Self { self.color = c; self }
    pub fn border_color(mut self, c: UColor) -> Self { self.border_color = c; self }
    pub fn border_thickness(mut self, t: f32) -> Self { self.border_thickness = t; self }
    pub fn corner_radius(mut self, r: f32) -> Self { self.corner_radius = r; self }
    pub fn on_click(mut self, callback: impl FnMut() + Send + 'static) -> Self {
        self.on_click = Some(Box::new(callback));
        self
    }
}

impl Widget for Button {
    fn min_size(&self, _ctx: &mut dyn LayoutContext) -> Size { Size::new(80.0, 30.0) }
    fn margin(&self) -> EdgeInsets { self.margin }
    fn padding(&self) -> EdgeInsets { self.padding }
    fn create_render_object(&mut self) -> Box<dyn RenderBox> {
        Box::new(ButtonRenderObject {
            text: self.text.clone(),
            font_name: self.font_name.clone(),
            padding: self.padding,
            margin: self.margin,
            color: self.color,
            border_color: self.border_color,
            border_thickness: self.border_thickness,
            radius: self.corner_radius,
            position: Point::default(),
            size: Size::default(),
            is_hovered: false, is_focused: false, is_pressed: false, is_dragging: false,
            id: None, on_click: self.on_click.take(),
            cached_bg_vertices: Vec::new(), cached_bg_indices: Vec::new(),
            cached_border_vertices: Vec::new(), cached_border_indices: Vec::new(),
            dirty: true,
        })
    }
}
impl LeafRenderObjectWidget for Button {}

struct ButtonRenderObject {
    text: String, font_name: String, padding: EdgeInsets, margin: EdgeInsets,
    color: UColor, border_color: UColor, border_thickness: f32, radius: f32,
    position: Point, size: Size,
    is_hovered: bool, is_focused: bool, is_pressed: bool, is_dragging: bool,
    id: Option<WidgetId>, on_click: Option<Box<dyn FnMut() + Send>>,
    cached_bg_vertices: Vec<Vertex>, cached_bg_indices: Vec<u32>,
    cached_border_vertices: Vec<Vertex>, cached_border_indices: Vec<u32>,
    dirty: bool,
}

impl ButtonRenderObject {
    fn current_color(&self) -> UColor {
        if self.is_dragging { UColor([0.5, 0.5, 0.5, 1.0]) }
        else if self.is_pressed { UColor([0.1, 0.2, 0.4, 1.0]) }
        else if self.is_hovered || self.is_focused { UColor([0.3, 0.4, 0.6, 1.0]) }
        else { self.color }
    }
    fn mark_dirty(&mut self) { self.dirty = true; }

    fn rebuild_cache(&mut self, primitives: &dyn Primitives) {
        if self.size.width < 1.0 || self.size.height < 1.0 { return; }
        let rect = Rect::new(0.0, 0.0, self.size.width, self.size.height);
        
        let bg_c = self.current_color();
        if bg_c.0[3] > 0.0 {
            let (v, i) = primitives.rounded_rect_vertices_indices(rect, self.radius, bg_c);
            self.cached_bg_vertices = v;
            self.cached_bg_indices = i;
        } else {
            self.cached_bg_vertices.clear(); self.cached_bg_indices.clear();
        }

        if self.border_thickness > 0.0 && self.border_color.0[3] > 0.0 {
            let (v, i) = primitives.rounded_rect_outline_vertices_indices(rect, self.radius, self.border_thickness, self.border_color);
            self.cached_border_vertices = v;
            self.cached_border_indices = i;
        } else {
            self.cached_border_vertices.clear(); self.cached_border_indices.clear();
        }
        self.dirty = false;
    }
}

impl RenderBox for ButtonRenderObject {
    fn layout(&mut self, constraints: Constraints, ctx: &mut dyn LayoutContext) -> Size {
        let text_size = ctx.measure_text_with_font(&self.font_name, &self.text, 16.0, constraints.max_width);
        let inner = Size::new(text_size.width + self.padding.left + self.padding.right,
                              text_size.height + self.padding.top + self.padding.bottom);
        let new_size = constraints.constrain(inner);
        if (new_size.width - self.size.width).abs() > 0.01 || (new_size.height - self.size.height).abs() > 0.01 {
            self.size = new_size;
            self.mark_dirty();
        }
        new_size
    }
    fn set_position(&mut self, pos: Point) {
        if self.position != pos { self.position = pos; self.mark_dirty(); }
    }
    fn position(&self) -> Point { self.position }
    fn size(&self) -> Size { self.size }

    fn render(&mut self, ctx: &mut RenderContext) {
        if self.dirty { self.rebuild_cache(ctx.primitives); }
        
        // Фон
        let mut world_bg = self.cached_bg_vertices.clone();
        for v in &mut world_bg {
            v.position[0] += self.position.x;
            v.position[1] += self.position.y;
        }
        if !world_bg.is_empty() {
            ctx.add_command(0, world_bg, self.cached_bg_indices.clone());
        }

        // Граница
        let mut world_border = self.cached_border_vertices.clone();
        for v in &mut world_border {
            v.position[0] += self.position.x;
            v.position[1] += self.position.y;
        }
        if !world_border.is_empty() {
            ctx.add_command(0, world_border, self.cached_border_indices.clone());
        }

        // Текст
        if let Some(font) = ctx.font_system.get_font(&self.font_name) {
            let (verts, inds) = ctx.font_system.generate_text_vertices_with_font(
                font, &self.text,
                self.position.x + self.padding.left, self.position.y + self.padding.top,
                16.0, UColor([1.0, 1.0, 1.0, 1.0]), ctx.primitives,
            );
            ctx.add_command(font.texture_id(), verts, inds);
        }
    }

    fn children(&self) -> &[Box<dyn RenderBox>] { &[] }
    fn children_mut(&mut self) -> &mut [Box<dyn RenderBox>] { &mut [] }
    fn hit_test(&self, point: Point) -> bool {
        Rect::new(self.position.x, self.position.y, self.size.width, self.size.height).contains(point)
    }
    fn handle_event(&mut self, event: &Event, _ui: &mut UiManager) -> bool {
        match event {
            Event::Click(_) => { if let Some(cb) = &mut self.on_click { cb(); } true }
            Event::PointerDown(_) => { self.is_pressed = true; self.mark_dirty(); true }
            Event::PointerUp(_) => { self.is_pressed = false; self.mark_dirty(); true }
            Event::PointerMove(point) => {
                let inside = self.hit_test(*point);
                if inside != self.is_hovered { self.is_hovered = inside; self.mark_dirty(); }
                true
            }
            _ => false,
        }
    }
    fn can_focus(&self) -> bool { true }
    fn set_focused(&mut self, f: bool) { if self.is_focused != f { self.is_focused = f; self.mark_dirty(); } }
    fn is_focused(&self) -> bool { self.is_focused }
    fn handle_key_down(&mut self, key: Key, _mod: KeyboardModifiers) -> bool {
        if key == Key::Enter || key == Key::Space { if let Some(cb) = &mut self.on_click { cb(); } true } else { false }
    }
    fn handle_char_input(&mut self, ch: char) -> bool {
        if ch == ' ' { if let Some(cb) = &mut self.on_click { cb(); } true } else { false }
    }
    fn can_drag(&self) -> bool { true }
    fn drag_data(&self) -> Option<DragData> { Some(DragData::Text(self.text.clone())) }
    fn on_drag_start(&mut self, _p: Point) { self.is_dragging = true; self.mark_dirty(); self.is_hovered = false; }
    fn on_drag_end(&mut self, _c: bool) { self.is_dragging = false; self.mark_dirty(); }
    fn can_drop(&self, d: &DragData) -> bool { matches!(d, DragData::Text(_)) }
    fn on_drag_enter(&mut self, d: &DragData, _p: Point) { if let DragData::Text(s) = d { eprintln!("Button drag enter: {}", s); } self.is_hovered = true; self.mark_dirty(); }
    fn on_drag_leave(&mut self) { self.is_hovered = false; self.mark_dirty(); }
    fn on_drop(&mut self, d: &DragData, _p: Point) { if let DragData::Text(s) = d { eprintln!("Button drop: {}", s); } }
    fn widget_id(&self) -> Option<WidgetId> { self.id }
    fn margin(&self) -> EdgeInsets { self.margin }
}
impl Drop for ButtonRenderObject { fn drop(&mut self) {} }