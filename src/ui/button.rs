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
            corner_radius: 4.0,
            on_click: None,
        }
    }
    pub fn padding(mut self, p: EdgeInsets) -> Self { self.padding = p; self }
    pub fn margin(mut self, m: EdgeInsets) -> Self { self.margin = m; self }
    pub fn color(mut self, c: UColor) -> Self { self.color = c; self }
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
            radius: self.corner_radius,
            position: Point::default(),
            size: Size::default(),
            is_hovered: false,
            is_focused: false,
            is_pressed: false,
            is_dragging: false,
            id: None,
            on_click: self.on_click.take(),
            // Кэш
            cached_vertices: Vec::new(),
            cached_indices: Vec::new(),
            dirty: true,
        })
    }
}

impl LeafRenderObjectWidget for Button {}

struct ButtonRenderObject {
    text: String,
    font_name: String,
    padding: EdgeInsets,
    margin: EdgeInsets,
    color: UColor,
    radius: f32,
    position: Point,
    size: Size,
    is_hovered: bool,
    is_focused: bool,
    is_pressed: bool,
    is_dragging: bool,
    id: Option<WidgetId>,
    on_click: Option<Box<dyn FnMut() + Send>>,
    // Кэш
    cached_vertices: Vec<Vertex>,
    cached_indices: Vec<u32>,
    dirty: bool,
}

impl ButtonRenderObject {
    fn current_color(&self) -> UColor {
        if self.is_dragging {
            UColor([0.5, 0.5, 0.5, 1.0])
        } else if self.is_pressed {
            UColor([0.1, 0.2, 0.4, 1.0])
        } else if self.is_hovered || self.is_focused {
            UColor([0.3, 0.4, 0.6, 1.0])
        } else {
            self.color
        }
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn rebuild_cache(&mut self, primitives: &dyn Primitives) {
        let rect = Rect::new(0.0, 0.0, self.size.width, self.size.height);
        let (verts, inds) = primitives.rounded_rect_vertices_indices(rect, self.radius, self.current_color());
        self.cached_vertices = verts;
        self.cached_indices = inds;
        self.dirty = false;
    }
}

impl RenderBox for ButtonRenderObject {
    fn layout(&mut self, constraints: Constraints, ctx: &mut dyn LayoutContext) -> Size {
        let text_size = ctx.measure_text_with_font(&self.font_name, &self.text, 16.0, constraints.max_width);
        let inner_size = Size::new(
            text_size.width + self.padding.left + self.padding.right,
            text_size.height + self.padding.top + self.padding.bottom,
        );
        let new_size = constraints.constrain(inner_size);
        if (new_size.width - self.size.width).abs() > 0.01 || (new_size.height - self.size.height).abs() > 0.01 {
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
        // Фон (с кэшем)
        if self.dirty {
            self.rebuild_cache(ctx.primitives);
        }
        // Сдвигаем вершины в мировые координаты (позиция виджета)
        let mut world_verts = self.cached_vertices.clone();
        for v in &mut world_verts {
            v.position[0] += self.position.x;
            v.position[1] += self.position.y;
        }
        ctx.add_command(0, world_verts, self.cached_indices.clone());

        // Текст (генерируем каждый раз, т.к. позиция текста зависит от позиции кнопки и отступов)
        if let Some(font) = ctx.font_system.get_font(&self.font_name) {
            let (verts, inds) = ctx.font_system.generate_text_vertices_with_font(
                font,
                &self.text,
                self.position.x + self.padding.left,
                self.position.y + self.padding.top,
                16.0,
                UColor([1.0, 1.0, 1.0, 1.0]),
                ctx.primitives,
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
            Event::Click(_) => {
                if let Some(cb) = &mut self.on_click { cb(); }
                true
            }
            Event::PointerDown(_) => {
                self.is_pressed = true;
                self.mark_dirty();
                true
            }
            Event::PointerUp(_) => {
                self.is_pressed = false;
                self.mark_dirty();
                true
            }
            Event::PointerMove(point) => {
                let inside = self.hit_test(*point);
                if inside != self.is_hovered {
                    self.is_hovered = inside;
                    self.mark_dirty();
                }
                true
            }
            _ => false,
        }
    }

    fn can_focus(&self) -> bool { true }
    fn set_focused(&mut self, focused: bool) {
        if self.is_focused != focused {
            self.is_focused = focused;
            self.mark_dirty();
        }
    }
    fn is_focused(&self) -> bool { self.is_focused }

    fn handle_key_down(&mut self, key: Key, _mod: KeyboardModifiers) -> bool {
        if key == Key::Enter || key == Key::Space {
            if let Some(cb) = &mut self.on_click { cb(); }
            true
        } else {
            false
        }
    }
    fn handle_char_input(&mut self, ch: char) -> bool {
        if ch == ' ' {
            if let Some(cb) = &mut self.on_click { cb(); }
            true
        } else {
            false
        }
    }

    fn can_drag(&self) -> bool { true }
    fn drag_data(&self) -> Option<DragData> {
        Some(DragData::Text(self.text.clone()))
    }
    fn on_drag_start(&mut self, _point: Point) {
        self.is_dragging = true;
        self.mark_dirty();
        self.is_hovered = false;
    }
    fn on_drag_end(&mut self, _cancelled: bool) {
        self.is_dragging = false;
        self.mark_dirty();
    }

    fn can_drop(&self, data: &DragData) -> bool {
        matches!(data, DragData::Text(_))
    }
    fn on_drag_enter(&mut self, data: &DragData, _point: Point) {
        if let DragData::Text(s) = data { eprintln!("Button drag enter: {}", s); }
        self.is_hovered = true;
        self.mark_dirty();
    }
    fn on_drag_leave(&mut self) {
        self.is_hovered = false;
        self.mark_dirty();
    }
    fn on_drop(&mut self, data: &DragData, _point: Point) {
        if let DragData::Text(s) = data { eprintln!("Button drop: {}", s); }
    }

    fn widget_id(&self) -> Option<WidgetId> { self.id }
    fn margin(&self) -> EdgeInsets { self.margin }
}

impl Drop for ButtonRenderObject {
    fn drop(&mut self) {}
}