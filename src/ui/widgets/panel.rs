// src/ui/widgets/panel.rs
use super::widget::Widget;
use crate::common::render_box::{RenderBox, WidgetId};
use crate::common::render_context::RenderContext;
use crate::common::primitives::Primitives;
use crate::common::types::*;
use crate::common::vertex::Vertex;
use crate::common::event::{Event, KeyboardModifiers, DragData};
use crate::common::key::Key;
use crate::ui_manager::UiManager;
use crate::texture_manager::{TextureManager, SamplerKind};

pub struct Panel {
    id: Option<WidgetId>,
    child: Box<dyn Widget>,
    background_color: UColor,
    corner_radius: f32,
    margin: EdgeInsets,
    padding: EdgeInsets,
    // Текстурный оверлей (рисуется ПОВЕРХ фона)
    overlay_texture_id: Option<u64>,
    overlay_fit: BackgroundFit,
    overlay_tint: UColor,
}

impl Panel {
    pub fn new(child: Box<dyn Widget>) -> Self {
        Self {
            id: None,
            child,
            background_color: UColor([0.2, 0.2, 0.2, 1.0]),
            corner_radius: 8.0,
            margin: EdgeInsets::default(),
            padding: EdgeInsets::all(12.0),
            overlay_texture_id: None,
            overlay_fit: BackgroundFit::Stretch,
            overlay_tint: UColor([1.0, 1.0, 1.0, 1.0]),
        }
    }

    pub fn background(mut self, color: UColor) -> Self { self.background_color = color; self }
    pub fn corner_radius(mut self, radius: f32) -> Self { self.corner_radius = radius; self }
    pub fn margin(mut self, margin: EdgeInsets) -> Self { self.margin = margin; self }
    pub fn padding(mut self, padding: EdgeInsets) -> Self { self.padding = padding; self }

    pub fn background_texture_overlay(mut self, id: u64, fit: BackgroundFit, tint: Option<UColor>) -> Self {
        self.overlay_texture_id = Some(id);
        self.overlay_fit = fit;
        if let Some(c) = tint { self.overlay_tint = c; }
        self
    }
    
    pub fn with_id(mut self, id: WidgetId) -> Self {
        self.id = Some(id);
        self
    }
}

impl Widget for Panel {
    fn min_size(&self, ctx: &mut dyn LayoutContext) -> Size {
        let child_min = self.child.min_size(ctx);
        let padded = self.padding.inflate(child_min);
        self.margin.inflate(padded)
    }
    fn margin(&self) -> EdgeInsets { self.margin }
    fn padding(&self) -> EdgeInsets { self.padding }

    fn create_render_object(&mut self) -> Box<dyn RenderBox> {
        let child_render = self.child.create_render_object();
        Box::new(PanelRenderObject {
            id: self.id,            
            child: child_render,
            background_color: self.background_color,
            corner_radius: self.corner_radius,
            padding: self.padding,
            margin: self.margin,
            position: Point::default(),
            size: Size::default(),
            overlay_texture_id: self.overlay_texture_id,
            overlay_fit: self.overlay_fit,
            overlay_tint: self.overlay_tint,
            cached_bg_vertices: Vec::new(),
            cached_bg_indices: Vec::new(),
            cached_overlay_vertices: Vec::new(),
            cached_overlay_indices: Vec::new(),
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

struct PanelRenderObject {
    id: Option<WidgetId>,
    child: Box<dyn RenderBox>,
    background_color: UColor,
    corner_radius: f32,
    padding: EdgeInsets,
    margin: EdgeInsets,
    position: Point,
    size: Size,
    overlay_texture_id: Option<u64>,
    overlay_fit: BackgroundFit,
    overlay_tint: UColor,
    cached_bg_vertices: Vec<Vertex>,
    cached_bg_indices: Vec<u32>,
    cached_overlay_vertices: Vec<Vertex>,
    cached_overlay_indices: Vec<u32>,
    dirty: bool,
}

impl PanelRenderObject {
    fn mark_dirty(&mut self) { self.dirty = true; }

    fn rebuild_cache(&mut self, primitives: &dyn Primitives, textures: &TextureManager) {
        let rect = Rect::new(
            self.position.x + self.margin.left,
            self.position.y + self.margin.top,
            self.size.width - self.margin.left - self.margin.right,
            self.size.height - self.margin.top - self.margin.bottom,
        );

        // 1️⃣ Слой фона (Solid)
        if self.background_color.0[3] > 0.0 {
            let (v, i) = primitives.rounded_rect_vertices_indices(rect, self.corner_radius, self.background_color);
            self.cached_bg_vertices = v;
            self.cached_bg_indices = i;
        } else {
            self.cached_bg_vertices.clear();
            self.cached_bg_indices.clear();
        }

        // 2️⃣ Слой оверлея (PNG)
        if let Some(oid) = self.overlay_texture_id {
            if let Some(tex_size) = textures.get_size_by_id(oid) {
                let tc = self.overlay_fit.calculate_tex_coords(&rect, tex_size);
                let (v, i) = primitives.rounded_textured_rect_vertices_indices(
                    rect, self.corner_radius, tc, self.overlay_tint
                );
                self.cached_overlay_vertices = v;
                self.cached_overlay_indices = i;
            }
        } else {
            self.cached_overlay_vertices.clear();
            self.cached_overlay_indices.clear();
        }
        self.dirty = false;
    }
}

impl RenderBox for PanelRenderObject {
    fn layout(&mut self, constraints: Constraints, ctx: &mut dyn LayoutContext) -> Size {
        let total_horizontal = self.margin.left + self.margin.right + self.padding.left + self.padding.right;
        let total_vertical = self.margin.top + self.margin.bottom + self.padding.top + self.padding.bottom;
        let inner_constraints = Constraints {
            min_width: (constraints.min_width - total_horizontal).max(0.0),
            max_width: (constraints.max_width - total_horizontal).max(0.0),
            min_height: (constraints.min_height - total_vertical).max(0.0),
            max_height: (constraints.max_height - total_vertical).max(0.0),
        };
        let child_size = self.child.layout(inner_constraints, ctx);
        let total_size = Size::new(child_size.width + total_horizontal, child_size.height + total_vertical);
        let new_size = constraints.constrain(total_size);
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
        if self.dirty {
            self.rebuild_cache(ctx.primitives, ctx.textures);
        }

        // 🎨 Сначала заливаем цвет фона (текстура 0, сэмплер Clamp)
        if !self.cached_bg_vertices.is_empty() {
            let mut world_bg = self.cached_bg_vertices.clone();
            for v in &mut world_bg {
                v.position[0] += self.position.x;
                v.position[1] += self.position.y;
            }
            ctx.add_command(0, SamplerKind::Clamp, world_bg, self.cached_bg_indices.clone());
        }

        // 🖼️ Затем накладываем PNG-оверлей (с выбором сэмплера в зависимости от fit)
        if !self.cached_overlay_vertices.is_empty() {
            if let Some(oid) = self.overlay_texture_id {
                let sampler_kind = match self.overlay_fit {
                    BackgroundFit::Tile { .. } => SamplerKind::Repeat,
                    _ => SamplerKind::Clamp,
                };
                let mut world_overlay = self.cached_overlay_vertices.clone();
                for v in &mut world_overlay {
                    v.position[0] += self.position.x;
                    v.position[1] += self.position.y;
                }
                ctx.add_command(oid, sampler_kind, world_overlay, self.cached_overlay_indices.clone());
            }
        }

        // 📦 Контент поверх обоих слоёв
        self.child.render(ctx);
    }

    fn children(&self) -> &[Box<dyn RenderBox>] { std::slice::from_ref(&self.child) }
    fn children_mut(&mut self) -> &mut [Box<dyn RenderBox>] { std::slice::from_mut(&mut self.child) }

    fn hit_test(&self, point: Point) -> bool {
        Rect::new(
            self.position.x + self.margin.left,
            self.position.y + self.margin.top,
            self.size.width - self.margin.left - self.margin.right,
            self.size.height - self.margin.top - self.margin.bottom,
        ).contains(point)
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

    fn widget_id(&self) -> Option<WidgetId> {
        self.id
    }
    fn set_widget_id(&mut self, id: WidgetId) {
        self.id = Some(id);
    }

}