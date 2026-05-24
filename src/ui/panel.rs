// src/widgets/panel.rs
use super::widget::Widget;
use crate::common::render_box::RenderBox;
use crate::common::render_context::RenderContext;
use crate::common::primitives::Primitives;
use crate::common::{Vertex, types::*};
use crate::common::event::{Event, KeyboardModifiers, DragData};
use crate::common::key::Key;
use crate::ui_manager::UiManager;
use crate::texture_manager::TextureManager;

pub struct Panel {
    child: Box<dyn Widget>,
    background_color: UColor,
    background_texture_id: Option<u64>,
    bg_fit: BackgroundFit,
    bg_tint: UColor,
    corner_radius: f32,
    margin: EdgeInsets,
    padding: EdgeInsets,
}

impl Panel {
    pub fn new(child: Box<dyn Widget>) -> Self {
        Self {
            child,
            background_color: UColor([0.2, 0.2, 0.2, 1.0]),
            background_texture_id: None,
            bg_fit: BackgroundFit::Stretch,
            bg_tint: UColor([1.0, 1.0, 1.0, 1.0]),
            corner_radius: 8.0,
            margin: EdgeInsets::default(),
            padding: EdgeInsets::all(12.0),
        }
    }

    pub fn background(mut self, color: UColor) -> Self { self.background_color = color; self }
    
    /// Устанавливает текстуру фона, стратегию заполнения и опциональный цвет-тинт.
    pub fn background_texture(mut self, id: u64, fit: BackgroundFit, tint: Option<UColor>) -> Self {
        self.background_texture_id = Some(id);
        self.bg_fit = fit;
        if let Some(c) = tint { self.bg_tint = c; }
        self
    }
    
    pub fn corner_radius(mut self, radius: f32) -> Self { self.corner_radius = radius; self }
    pub fn margin(mut self, margin: EdgeInsets) -> Self { self.margin = margin; self }
    pub fn padding(mut self, padding: EdgeInsets) -> Self { self.padding = padding; self }
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
            child: child_render,
            background_color: self.background_color,
            background_texture_id: self.background_texture_id,
            bg_fit: self.bg_fit,
            bg_tint: self.bg_tint,
            corner_radius: self.corner_radius,
            padding: self.padding,
            margin: self.margin,
            position: Point::default(),
            size: Size::default(),
            id: None,
            cached_vertices: Vec::new(),
            cached_indices: Vec::new(),
            dirty: true,
        })
    }
}

struct PanelRenderObject {
    child: Box<dyn RenderBox>,
    background_color: UColor,
    background_texture_id: Option<u64>,
    bg_fit: BackgroundFit,
    bg_tint: UColor,
    corner_radius: f32,
    padding: EdgeInsets,
    margin: EdgeInsets,
    position: Point,
    size: Size,
    id: Option<u64>,
    cached_vertices: Vec<Vertex>,
    cached_indices: Vec<u32>,
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

        // 🖼️ Попытка использовать текстуру
        if let Some(tid) = self.background_texture_id {
            if let Some(tex_size) = textures.get_size_by_id(tid) {
                let tc = self.bg_fit.calculate_tex_coords(&rect, tex_size);
                let (v, i) = primitives.rounded_textured_rect_vertices_indices(
                    rect, self.corner_radius, tc, self.bg_tint
                );
                self.cached_vertices = v;
                self.cached_indices = i;
                self.dirty = false;
                return; // Успешно, выходим
            }
        }

        // 🎨 Fallback на сплошной цвет
        if self.background_color.0[3] > 0.0 {
            let (v, i) = primitives.rounded_rect_vertices_indices(rect, self.corner_radius, self.background_color);
            self.cached_vertices = v;
            self.cached_indices = i;
        } else {
            self.cached_vertices.clear();
            self.cached_indices.clear();
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
        if self.position != pos {
            self.position = pos;
            self.mark_dirty();
        }
    }
    fn position(&self) -> Point { self.position }
    fn size(&self) -> Size { self.size }

    fn render(&mut self, ctx: &mut RenderContext) {
        if self.dirty {
            self.rebuild_cache(ctx.primitives, ctx.textures);
        }

        if !self.cached_vertices.is_empty() {
            // Определяем актуальный ID текстуры для draw call
            let tex_id = if let Some(tid) = self.background_texture_id {
                // Если текстура загрузилась между вызовами rebuild_cache, используем её ID
                if ctx.textures.get_size_by_id(tid).is_some() { tid } else { 0 }
            } else {
                0
            };
            ctx.add_command(tex_id, self.cached_vertices.clone(), self.cached_indices.clone());
        }
        
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
        if let Some(point) = event.point() {
            if !self.hit_test(point) { return false; }
        }
        self.child.handle_event(event, ui_manager)
    }

    fn can_focus(&self) -> bool { self.child.can_focus() }
    fn set_focused(&mut self, focused: bool) { self.child.set_focused(focused); }
    fn is_focused(&self) -> bool { self.child.is_focused() }
    fn handle_key_down(&mut self, key: Key, mods: KeyboardModifiers) -> bool { self.child.handle_key_down(key, mods) }
    fn handle_key_up(&mut self, key: Key, mods: KeyboardModifiers) -> bool { self.child.handle_key_up(key, mods) }
    fn handle_char_input(&mut self, ch: char) -> bool { self.child.handle_char_input(ch) }
    fn can_drag(&self) -> bool { self.child.can_drag() }
    fn drag_data(&self) -> Option<DragData> { self.child.drag_data() }
    fn on_drag_start(&mut self, point: Point) { self.child.on_drag_start(point); }
    fn on_drag_move(&mut self, point: Point) { self.child.on_drag_move(point); }
    fn on_drag_end(&mut self, cancelled: bool) { self.child.on_drag_end(cancelled); }
    fn can_drop(&self, data: &DragData) -> bool { self.child.can_drop(data) }
    fn on_drag_enter(&mut self, data: &DragData, point: Point) { self.child.on_drag_enter(data, point); }
    fn on_drag_leave(&mut self) { self.child.on_drag_leave(); }
    fn on_drop(&mut self, data: &DragData, point: Point) { self.child.on_drop(data, point); }
    fn widget_id(&self) -> Option<u64> { self.id }
}