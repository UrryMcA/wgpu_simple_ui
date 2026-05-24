// src/widgets/icon_button.rs
use super::widget::Widget;
use crate::common::render_box::RenderBox;
use crate::common::render_context::RenderContext;
use crate::common::primitives::Primitives;
use crate::common::{Vertex, types::*};
use crate::common::event::{Event, KeyboardModifiers, DragData};
use crate::common::key::Key;
use crate::ui_manager::UiManager;
use crate::texture_manager::TextureManager;
use crate::ui::Container;
use crate::ui::image::Image;
use crate::ui::label::Label;

/// Композитная кнопка с иконкой и текстом.
/// Фон рендерится отдельно (solid/texture + BackgroundFit), 
/// контент позиционируется через внутренний Container.
pub struct IconButton {
    container: Container,
    background_color: UColor,
    bg_texture_id: Option<u64>,
    bg_fit: BackgroundFit,
    bg_tint: UColor,
    border_color: UColor,
    border_thickness: f32,
    corner_radius: f32,
    margin: EdgeInsets,
    padding: EdgeInsets,
    on_click: Option<Box<dyn FnMut() + Send>>,
}

impl IconButton {
    /// Создаёт кнопку с иконкой слева и текстом справа.
    pub fn new(text: impl Into<String>, icon_texture_id: u64, icon_size: Size) -> Self {
        let mut container = Container::horizontal()
            .spacing(8.0)
            .alignment(Alignment::Center);

        let icon = Image::new(icon_texture_id, icon_size.width, icon_size.height);
        let label = Label::new(text)
            .font_size(16.0)
            .color(UColor([1.0, 1.0, 1.0, 1.0]));

        container = container.add_child(Box::new(icon));
        container = container.add_child(Box::new(label));

        Self {
            container,
            background_color: UColor([0.2, 0.3, 0.5, 1.0]),
            bg_texture_id: None,
            bg_fit: BackgroundFit::Stretch,
            bg_tint: UColor([1.0, 1.0, 1.0, 1.0]),
            border_color: UColor([0.0, 0.0, 0.0, 0.0]),
            border_thickness: 0.0,
            corner_radius: 6.0,
            margin: EdgeInsets::default(),
            padding: EdgeInsets::all(10.0),
            on_click: None,
        }
    }

    pub fn background(mut self, color: UColor) -> Self { self.background_color = color; self }
    pub fn background_texture(mut self, id: u64, fit: BackgroundFit, tint: Option<UColor>) -> Self {
        self.bg_texture_id = Some(id);
        self.bg_fit = fit;
        if let Some(c) = tint { self.bg_tint = c; }
        self
    }
    pub fn border_color(mut self, c: UColor) -> Self { self.border_color = c; self }
    pub fn border_thickness(mut self, t: f32) -> Self { self.border_thickness = t; self }
    pub fn corner_radius(mut self, r: f32) -> Self { self.corner_radius = r; self }
    pub fn margin(mut self, m: EdgeInsets) -> Self { self.margin = m; self }
    pub fn padding(mut self, p: EdgeInsets) -> Self { self.padding = p; self }
    pub fn on_click(mut self, cb: impl FnMut() + Send + 'static) -> Self {
        self.on_click = Some(Box::new(cb));
        self
    }
}

impl Widget for IconButton {
    fn min_size(&self, ctx: &mut dyn LayoutContext) -> Size {
        let child_min = self.container.min_size(ctx);
        let padded = self.padding.inflate(child_min);
        self.margin.inflate(padded)
    }
    fn margin(&self) -> EdgeInsets { self.margin }
    fn padding(&self) -> EdgeInsets { self.padding }

    fn create_render_object(&mut self) -> Box<dyn RenderBox> {
        let child_render = self.container.create_render_object();
        Box::new(IconButtonRenderObject {
            child: child_render,
            background_color: self.background_color,
            bg_texture_id: self.bg_texture_id,
            bg_fit: self.bg_fit,
            bg_tint: self.bg_tint,
            border_color: self.border_color,
            border_thickness: self.border_thickness,
            corner_radius: self.corner_radius,
            padding: self.padding,
            margin: self.margin,
            position: Point::default(),
            size: Size::default(),
            is_hovered: false, is_focused: false, is_pressed: false, is_dragging: false,
            id: None,
            on_click: self.on_click.take(),
            cached_bg_vertices: Vec::new(),
            cached_bg_indices: Vec::new(),
            cached_border_vertices: Vec::new(),
            cached_border_indices: Vec::new(),
            dirty: true,
        })
    }
}

struct IconButtonRenderObject {
    child: Box<dyn RenderBox>,
    background_color: UColor,
    bg_texture_id: Option<u64>,
    bg_fit: BackgroundFit,
    bg_tint: UColor,
    border_color: UColor,
    border_thickness: f32,
    corner_radius: f32,
    padding: EdgeInsets,
    margin: EdgeInsets,
    position: Point,
    size: Size,
    is_hovered: bool, is_focused: bool, is_pressed: bool, is_dragging: bool,
    id: Option<u64>,
    on_click: Option<Box<dyn FnMut() + Send>>,
    cached_bg_vertices: Vec<Vertex>,
    cached_bg_indices: Vec<u32>,
    cached_border_vertices: Vec<Vertex>,
    cached_border_indices: Vec<u32>,
    dirty: bool,
}

impl IconButtonRenderObject {
    fn mark_dirty(&mut self) { self.dirty = true; }

    fn effective_tint(&self) -> UColor {
        let mut c = self.bg_tint.0;
        if self.is_dragging {
            c[3] *= 0.7;
        } else if self.is_pressed {
            c[0] *= 0.8; c[1] *= 0.8; c[2] *= 0.8;
        } else if self.is_hovered || self.is_focused {
            c[0] = (c[0] * 1.1).min(1.0);
            c[1] = (c[1] * 1.1).min(1.0);
            c[2] = (c[2] * 1.1).min(1.0);
        }
        UColor(c)
    }

    fn rebuild_cache(&mut self, primitives: &dyn Primitives, textures: &TextureManager) {
        let rect = Rect::new(
            0.0, 0.0,
            self.size.width - self.margin.left - self.margin.right,
            self.size.height - self.margin.top - self.margin.bottom,
        );
        let tint = self.effective_tint();

        // 🖼️ Попытка использовать текстуру
        if let Some(tid) = self.bg_texture_id {
            if let Some(tex_size) = textures.get_size_by_id(tid) {
                let tc = self.bg_fit.calculate_tex_coords(&rect, tex_size);
                let (v, i) = primitives.rounded_textured_rect_vertices_indices(
                    rect, self.corner_radius, tc, tint
                );
                self.cached_bg_vertices = v;
                self.cached_bg_indices = i;
                self.dirty = false;
                return;
            }
        }

        // 🎨 Fallback на сплошной цвет
        if self.background_color.0[3] > 0.0 {
            let (v, i) = primitives.rounded_rect_vertices_indices(rect, self.corner_radius, tint);
            self.cached_bg_vertices = v;
            self.cached_bg_indices = i;
        } else {
            self.cached_bg_vertices.clear();
            self.cached_bg_indices.clear();
        }
        self.dirty = false;
    }
}

impl RenderBox for IconButtonRenderObject {
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
        let new_size = constraints.constrain(Size::new(
            child_size.width + total_h,
            child_size.height + total_v,
        ));

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

        let tex_id = if let Some(tid) = self.bg_texture_id {
            if ctx.textures.get_size_by_id(tid).is_some() { tid } else { 0 }
        } else { 0 };

        // Рисуем фон
        let mut bg_verts = self.cached_bg_vertices.clone();
        for v in &mut bg_verts {
            v.position[0] += self.position.x + self.margin.left;
            v.position[1] += self.position.y + self.margin.top;
        }
        if !bg_verts.is_empty() {
            ctx.add_command(tex_id, bg_verts, self.cached_bg_indices.clone());
        }

        // Рисуем контент (иконка + текст)
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
        let inside = event.point().map(|p| self.hit_test(p)).unwrap_or(true);

        match event {
            Event::PointerMove(p) => {
                if inside != self.is_hovered {
                    self.is_hovered = inside;
                    self.mark_dirty();
                }
            }
            Event::PointerDown(_) => {
                self.is_pressed = true;
                self.mark_dirty();
            }
            Event::PointerUp(_) => {
                self.is_pressed = false;
                self.mark_dirty();
            }
            Event::Click(_) => {
                if inside {
                    if let Some(cb) = &mut self.on_click { cb(); }
                    return true;
                }
            }
            _ => {}
        }

        // Делегируем остальное детям
        self.child.handle_event(event, ui_manager)
    }

    fn can_focus(&self) -> bool { true }
    fn set_focused(&mut self, f: bool) {
        if self.is_focused != f { self.is_focused = f; self.mark_dirty(); }
    }
    fn is_focused(&self) -> bool { self.is_focused }

    fn handle_key_down(&mut self, key: Key, _mods: KeyboardModifiers) -> bool {
        if key == Key::Enter || key == Key::Space {
            if let Some(cb) = &mut self.on_click { cb(); }
            true
        } else { false }
    }

    fn can_drag(&self) -> bool { true }
    fn drag_data(&self) -> Option<DragData> { self.child.drag_data() }
    fn on_drag_start(&mut self, _p: Point) { self.is_dragging = true; self.mark_dirty(); self.is_hovered = false; }
    fn on_drag_end(&mut self, _c: bool) { self.is_dragging = false; self.mark_dirty(); }
    fn can_drop(&self, d: &DragData) -> bool { self.child.can_drop(d) }
    fn on_drag_enter(&mut self, d: &DragData, p: Point) { self.is_hovered = true; self.mark_dirty(); self.child.on_drag_enter(d, p); }
    fn on_drag_leave(&mut self) { self.is_hovered = false; self.mark_dirty(); self.child.on_drag_leave(); }
    fn on_drop(&mut self, d: &DragData, p: Point) { self.child.on_drop(d, p); }
    fn widget_id(&self) -> Option<u64> { self.id }
}