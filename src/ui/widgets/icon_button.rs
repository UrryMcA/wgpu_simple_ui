// src/widgets/icon_button.rs
use super::widget::{Widget, LeafRenderObjectWidget};
use crate::common::render_box::{RenderBox, WidgetId};
use crate::common::render_context::RenderContext;
use crate::common::{Primitives, Vertex, types::*};
use crate::common::event::{Event, KeyboardModifiers, DragData};
use crate::common::key::Key;
use crate::ui_manager::UiManager;
use crate::texture_manager::{SamplerKind, TextureManager};

pub struct IconButton {
    text: String,
    font_name: String,
    padding: EdgeInsets,
    margin: EdgeInsets,
    color: UColor,
    corner_radius: f32,
    on_click: Option<Box<dyn FnMut() + Send>>,
    // 🖼️ Прозрачный PNG-оверлей поверх залитого фона
    overlay_texture_id: Option<u64>,
    overlay_fit: BackgroundFit,
    overlay_tint: UColor,
    // 🎨 Иконка
    icon_texture_id: u64,
    icon_size: Size,
    icon_spacing: f32,
}

impl IconButton {
    pub fn new(text: impl Into<String>, icon_id: u64, icon_size: Size) -> Self {
        Self {
            text: text.into(),
            font_name: "default".into(),
            padding: EdgeInsets::all(10.0),
            margin: EdgeInsets::default(),
            color: UColor([0.2, 0.3, 0.5, 1.0]),
            corner_radius: 6.0,
            on_click: None,
            overlay_texture_id: None,
            overlay_fit: BackgroundFit::Stretch,
            overlay_tint: UColor([1.0, 1.0, 1.0, 1.0]),
            icon_texture_id: icon_id,
            icon_size,
            icon_spacing: 8.0,
        }
    }

    pub fn padding(mut self, p: EdgeInsets) -> Self { self.padding = p; self }
    pub fn margin(mut self, m: EdgeInsets) -> Self { self.margin = m; self }
    pub fn color(mut self, c: UColor) -> Self { self.color = c; self }
    pub fn corner_radius(mut self, r: f32) -> Self { self.corner_radius = r; self }
    pub fn icon_spacing(mut self, s: f32) -> Self { self.icon_spacing = s; self }
    pub fn on_click(mut self, cb: impl FnMut() + Send + 'static) -> Self {
        self.on_click = Some(Box::new(cb));
        self
    }

    /// Добавляет прозрачный PNG поверх залитого фона.
    /// `tint` управляет яркостью и прозрачностью оверлея (alpha-канал).
    pub fn background_texture_overlay(mut self, id: u64, fit: BackgroundFit, tint: Option<UColor>) -> Self {
        self.overlay_texture_id = Some(id);
        self.overlay_fit = fit;
        if let Some(c) = tint { self.overlay_tint = c; }
        self
    }
}

impl Widget for IconButton {
    fn min_size(&self, ctx: &mut dyn LayoutContext) -> Size {
        let text_size = ctx.measure_text_with_font(&self.font_name, &self.text, 16.0, f32::MAX);
        let w = self.padding.left + self.icon_size.width + self.icon_spacing + text_size.width + self.padding.right;
        let h = self.padding.top + self.icon_size.height.max(text_size.height) + self.padding.bottom;
        self.margin.inflate(Size::new(w, h))
    }
    fn margin(&self) -> EdgeInsets { self.margin }
    fn padding(&self) -> EdgeInsets { self.padding }

    fn create_render_object(&mut self) -> Box<dyn RenderBox> {
        Box::new(IconButtonRenderObject {
            text: self.text.clone(),
            font_name: self.font_name.clone(),
            padding: self.padding,
            margin: self.margin,
            color: self.color,
            radius: self.corner_radius,
            position: Point::default(),
            size: Size::default(),
            is_hovered: false, is_focused: false, is_pressed: false, is_dragging: false,
            id: None, on_click: self.on_click.take(),
            overlay_texture_id: self.overlay_texture_id,
            overlay_fit: self.overlay_fit,
            overlay_tint: self.overlay_tint,
            icon_texture_id: self.icon_texture_id,
            icon_size: self.icon_size,
            icon_spacing: self.icon_spacing,
            cached_bg_vertices: Vec::new(), cached_bg_indices: Vec::new(),
            cached_overlay_vertices: Vec::new(), cached_overlay_indices: Vec::new(),
            dirty: true,
        })
    }
}
impl LeafRenderObjectWidget for IconButton {}

struct IconButtonRenderObject {
    text: String, font_name: String, padding: EdgeInsets, margin: EdgeInsets,
    color: UColor, radius: f32,
    position: Point, size: Size,
    is_hovered: bool, is_focused: bool, is_pressed: bool, is_dragging: bool,
    id: Option<WidgetId>, on_click: Option<Box<dyn FnMut() + Send>>,
    overlay_texture_id: Option<u64>, overlay_fit: BackgroundFit, overlay_tint: UColor,
    icon_texture_id: u64, icon_size: Size, icon_spacing: f32,
    cached_bg_vertices: Vec<Vertex>, cached_bg_indices: Vec<u32>,
    cached_overlay_vertices: Vec<Vertex>, cached_overlay_indices: Vec<u32>,
    dirty: bool,
}

impl IconButtonRenderObject {
    fn current_color(&self) -> UColor {
        if self.is_dragging { UColor([0.5, 0.5, 0.5, 1.0]) }
        else if self.is_pressed { UColor([0.1, 0.2, 0.4, 1.0]) }
        else if self.is_hovered || self.is_focused { UColor([0.3, 0.4, 0.6, 1.0]) }
        else { self.color }
    }

    fn current_overlay_tint(&self) -> UColor {
        let mut c = self.overlay_tint.0;
        if self.is_pressed { c[3] *= 0.85; c[0] *= 0.9; c[1] *= 0.9; c[2] *= 0.9; }
        UColor(c)
    }

    fn mark_dirty(&mut self) { self.dirty = true; }

    fn rebuild_cache(&mut self, primitives: &dyn Primitives, textures: &TextureManager) {
        if self.size.width < 1.0 || self.size.height < 1.0 { return; }
        let rect = Rect::new(0.0, 0.0, self.size.width, self.size.height);

        // 1️⃣ Слой 1: Залитый фон (Solid)
        let bg_c = self.current_color();
        if bg_c.0[3] > 0.0 {
            let (v, i) = primitives.rounded_rect_vertices_indices(rect, self.radius, bg_c);
            self.cached_bg_vertices = v;
            self.cached_bg_indices = i;
        } else {
            self.cached_bg_vertices.clear(); self.cached_bg_indices.clear();
        }

        // 2️⃣ Слой 2: Прозрачный PNG-оверлей (Alpha Blending)
        if let Some(oid) = self.overlay_texture_id {
            if let Some(tex_size) = textures.get_size_by_id(oid) {
                let tc = self.overlay_fit.calculate_tex_coords(&rect, tex_size);
                let tint = self.current_overlay_tint();
                let (v, i) = primitives.rounded_textured_rect_vertices_indices(rect, self.radius, tc, tint);
                self.cached_overlay_vertices = v;
                self.cached_overlay_indices = i;
            }
        } else {
            self.cached_overlay_vertices.clear(); self.cached_overlay_indices.clear();
        }
        self.dirty = false;
    }
}

impl RenderBox for IconButtonRenderObject {
    fn layout(&mut self, constraints: Constraints, ctx: &mut dyn LayoutContext) -> Size {
        let text_size = ctx.measure_text_with_font(&self.font_name, &self.text, 16.0, constraints.max_width);
        let inner = Size::new(
            self.icon_size.width + self.icon_spacing + text_size.width + self.padding.left + self.padding.right,
            self.icon_size.height.max(text_size.height) + self.padding.top + self.padding.bottom,
        );
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
        if self.dirty { self.rebuild_cache(ctx.primitives, ctx.textures); }

        let px = self.position.x;
        let py = self.position.y;

        // 🎨 Слой 1: Solid Background
        let mut bg = self.cached_bg_vertices.clone();
        for v in &mut bg { v.position[0] += px; v.position[1] += py; }
        if !bg.is_empty() { 
            ctx.add_command(0, SamplerKind::Clamp, bg, self.cached_bg_indices.clone()); 
        }

        // 🖼️ Слой 2: Transparent PNG Overlay (рисуется ПОВЕРХ фона)
        let mut overlay = self.cached_overlay_vertices.clone();
        for v in &mut overlay { v.position[0] += px; v.position[1] += py; }
        if !overlay.is_empty() {
            if let Some(oid) = self.overlay_texture_id {
                // фон (текстура 0, Clamp)
                let sampler_kind = match self.overlay_fit {
                BackgroundFit::Tile { .. } => SamplerKind::Repeat,
                    _ => SamplerKind::Clamp,
                };
                ctx.add_command(oid, sampler_kind, overlay, self.cached_overlay_indices.clone());
            }
        }

        // 🖼️ Слой 3: Иконка
        let icon_x = px + self.padding.left;
        let icon_y = py + self.padding.top + (self.size.height - self.padding.top - self.padding.bottom - self.icon_size.height) * 0.5;
        let icon_rect = Rect::new(icon_x, icon_y, self.icon_size.width, self.icon_size.height);
        let (icon_verts, icon_inds) = ctx.primitives.textured_rect_vertices_indices(
            icon_rect, TexCoords::new(0.0, 0.0, 1.0, 1.0), UColor([1.0, 1.0, 1.0, 1.0])
        );
        ctx.add_command(self.icon_texture_id, SamplerKind::Clamp, icon_verts, icon_inds);

        // 📝 Слой 4: Текст
        if let Some(font) = ctx.font_system.get_font(&self.font_name) {
            let text_x = icon_x + self.icon_size.width + self.icon_spacing;
            let text_y = py + self.padding.top + (self.size.height - self.padding.top - self.padding.bottom - 16.0) * 0.5;
            let (verts, inds) = ctx.font_system.generate_text_vertices_with_font(
                font, &self.text, text_x, text_y, 16.0, UColor([1.0, 1.0, 1.0, 1.0]), ctx.primitives,
            );
            ctx.add_command(font.texture_id(), SamplerKind::Clamp, verts, inds);
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
    fn can_drag(&self) -> bool { true }
    fn drag_data(&self) -> Option<DragData> { Some(DragData::Text(self.text.clone())) }
    fn on_drag_start(&mut self, _p: Point) { self.is_dragging = true; self.mark_dirty(); self.is_hovered = false; }
    fn on_drag_end(&mut self, _c: bool) { self.is_dragging = false; self.mark_dirty(); }
    fn can_drop(&self, d: &DragData) -> bool { matches!(d, DragData::Text(_)) }
    fn on_drag_enter(&mut self, d: &DragData, _p: Point) { self.is_hovered = true; self.mark_dirty(); self.on_drag_leave(); }
    fn on_drag_leave(&mut self) { self.is_hovered = false; self.mark_dirty(); }
    fn on_drop(&mut self, d: &DragData, _p: Point) { if let DragData::Text(s) = d { eprintln!("IconButton drop: {}", s); } }
    fn widget_id(&self) -> Option<WidgetId> { self.id }
    fn margin(&self) -> EdgeInsets { self.margin }
}
impl Drop for IconButtonRenderObject { fn drop(&mut self) {} }