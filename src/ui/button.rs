// src/widgets/button.rs
use super::widget::{Widget, LeafRenderObjectWidget};
use crate::common::render_box::{RenderBox, WidgetId};
use crate::common::render_context::RenderContext;
use crate::common::types::*;
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
    fn min_size(&self) -> Size { Size::new(80.0, 30.0) }
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
}

impl ButtonRenderObject {
    fn current_color(&self) -> UColor {
        if self.is_dragging {
            UColor([0.5, 0.5, 0.5, 1.0])          // серый при перетаскивании
        } else if self.is_pressed {
            UColor([0.1, 0.2, 0.4, 1.0])          // тёмно-синий при нажатии
        } else if self.is_hovered || self.is_focused {
            UColor([0.3, 0.4, 0.6, 1.0])          // подсветка при наведении/фокусе
        } else {
            self.color                            // обычный цвет
        }
    }
}

impl RenderBox for ButtonRenderObject {
    fn layout(&mut self, constraints: Constraints, ctx: &mut dyn LayoutContext) -> Size {
        let text_size = ctx.measure_text_with_font(&self.font_name, &self.text, 16.0, constraints.max_width);
        let inner_size = Size::new(
            text_size.width + self.padding.left + self.padding.right,
            text_size.height + self.padding.top + self.padding.bottom,
        );
        let constrained = constraints.constrain(inner_size);
        self.size = constrained;
        constrained
    }

    fn set_position(&mut self, pos: Point) { self.position = pos; }
    fn position(&self) -> Point { self.position }
    fn size(&self) -> Size { self.size }

    fn render(&mut self, ctx: &mut RenderContext) {
        let rect = Rect::new(self.position.x, self.position.y, self.size.width, self.size.height);
        let bg = ctx.primitives.rounded_rect_vertices(rect, self.radius, self.current_color());
        ctx.add_command(0, bg);

        if let Some(font) = ctx.font_system.get_font(&self.font_name) {
            let verts = ctx.font_system.generate_text_vertices_with_font(
                font,
                &self.text,
                self.position.x + self.padding.left,
                self.position.y + self.padding.top,
                16.0,
                UColor([1.0, 1.0, 1.0, 1.0]),
                ctx.primitives,
            );
            ctx.add_command(font.texture_id(), verts);
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
                if let Some(cb) = &mut self.on_click {
                    cb();
                }
                true
            }
            Event::PointerDown(_) => {
                self.is_pressed = true;
                true
            }
            Event::PointerUp(_) => {
                self.is_pressed = false;
                true
            }
            Event::PointerMove(point) => {
                let inside = self.hit_test(*point);
                self.is_hovered = inside;
                true
            }
            _ => false,
        }
    }

    fn can_focus(&self) -> bool { true }
    fn set_focused(&mut self, focused: bool) { self.is_focused = focused; }
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

    // Drag & Drop (источник)
    fn can_drag(&self) -> bool { true }
    fn drag_data(&self) -> Option<DragData> {
        Some(DragData::Text(self.text.clone()))
    }
    fn on_drag_start(&mut self, _point: Point) {
        self.is_dragging = true;
        self.is_hovered = false;
    }
    fn on_drag_move(&mut self, _point: Point) {
        // не требуется
    }
    fn on_drag_end(&mut self, _cancelled: bool) {
        self.is_dragging = false;
    }

    // Drag & Drop (цель)
    fn can_drop(&self, data: &DragData) -> bool {
        matches!(data, DragData::Text(_))
    }
    fn on_drag_enter(&mut self, data: &DragData, _point: Point) {
        if let DragData::Text(s) = data {
            eprintln!("Button drag enter: {}", s);
        }
        self.is_hovered = true;
    }
    fn on_drag_leave(&mut self) {
        self.is_hovered = false;
    }
    fn on_drop(&mut self, data: &DragData, _point: Point) {
        if let DragData::Text(s) = data {
            eprintln!("Button drop: {}", s);
        }
    }

    fn widget_id(&self) -> Option<WidgetId> { self.id }
    fn margin(&self) -> EdgeInsets { self.margin }
}

impl Drop for ButtonRenderObject {
    fn drop(&mut self) {}
}