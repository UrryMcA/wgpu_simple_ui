// src/widgets/button.rs
use super::widget::{Widget, LeafRenderObjectWidget};
use crate::common::render_box::{RenderBox, WidgetId};
use crate::common::{Primitives, types::*};
use crate::common::vertex::DrawCommand;
use crate::common::event::{Event, KeyboardModifiers, DragData};
use crate::common::key::Key;
use crate::generate_text_vertices;
use crate::texture_manager::TextureManager;
use crate::ui::UiManager;

// ------------------------------------------------------------
// Публичный виджет Button (конфигурация)
// ------------------------------------------------------------
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

    pub fn padding(mut self, p: EdgeInsets) -> Self {
        self.padding = p;
        self
    }

    pub fn margin(mut self, m: EdgeInsets) -> Self {
        self.margin = m;
        self
    }

    pub fn color(mut self, c: UColor) -> Self {
        self.color = c;
        self
    }

    pub fn corner_radius(mut self, r: f32) -> Self {
        self.corner_radius = r;
        self
    }

    pub fn on_click(mut self, callback: impl FnMut() + Send + 'static) -> Self {
        self.on_click = Some(Box::new(callback));
        self
    }
}

impl Widget for Button {
    fn min_size(&self) -> Size {
        Size::new(80.0, 30.0)
    }

    fn margin(&self) -> EdgeInsets {
        self.margin
    }

    fn padding(&self) -> EdgeInsets {
        self.padding
    }

    fn create_render_object(&self) -> Box<dyn RenderBox> {
        // Переносим колбэк в render object; в виджете он больше не нужен.
        let on_click = self.on_click.take();
        Box::new(ButtonRenderObject {
            text: self.text.clone(),
            font_name: self.font_name.clone(),
            padding: self.padding,
            color: self.color,
            radius: self.corner_radius,
            position: Point::default(),
            size: Size::default(),
            is_hovered: false,
            is_focused: false,
            id: None,
            on_click,
        })
    }
}

impl LeafRenderObjectWidget for Button {}

// ------------------------------------------------------------
// Объект рендеринга Button
// ------------------------------------------------------------
struct ButtonRenderObject {
    text: String,
    font_name: String,
    padding: EdgeInsets,
    color: UColor,
    radius: f32,
    position: Point,
    size: Size,
    // Состояния интерактивности
    is_hovered: bool,
    is_focused: bool,
    id: Option<WidgetId>,
    on_click: Option<Box<dyn FnMut() + Send>>,
}

impl ButtonRenderObject {
    /// Вспомогательный метод для получения текущего цвета фона в зависимости от состояния.
    fn current_color(&self) -> UColor {
        if self.is_pressed() {
            UColor([0.1, 0.2, 0.4, 1.0])
        } else if self.is_hovered || self.is_focused {
            UColor([0.3, 0.4, 0.6, 1.0])
        } else {
            self.color
        }
    }

    /// Заглушка для состояния нажатия (в реальном проекте нужно отслеживать PointerDown/Up).
    fn is_pressed(&self) -> bool {
        false
    }
}

impl RenderBox for ButtonRenderObject {
    // ---------- Обязательные методы ----------
    fn layout(&mut self, constraints: Constraints, ctx: &mut dyn LayoutContext) -> Size {
        // Регистрируем виджет, если ещё не зарегистрирован
        if self.id.is_none() {
            //self.id = Some(ui_manager.register_widget(self));
        }

        let inner = Size::new(80.0, 30.0);
        let total = self.padding.inflate(inner);
        let s = constraints.constrain(total);
        self.size = s;
        s
    }

    fn set_position(&mut self, pos: Point) {
        self.position = pos;
    }

    fn position(&self) -> Point {
        self.position
    }

    fn size(&self) -> Size {
        self.size
    }

    fn render(
        &self,
        commands: &mut Vec<DrawCommand>,
        primitives: &dyn Primitives,
        _textures: &TextureManager,
        ui_manager: &UiManager,
    ) {
        let rect = Rect::new(
            self.position.x,
            self.position.y,
            self.size.width,
            self.size.height,
        );

        // Фон с учётом состояния
        let bg_color = self.current_color();
        let bg = primitives.rounded_rect_vertices(rect, self.radius, bg_color);
        commands.push(DrawCommand {
            texture_id: 0,
            vertices: bg,
        });

        // Текст
        if let Some(font) = ui_manager.get_font(&self.font_name) {
            let tv = generate_text_vertices(
                font,
                &self.text,
                rect.x + self.padding.left,
                rect.y + self.padding.top,
                1.0,
                [1.0, 1.0, 1.0, 1.0],
                primitives,
            );
            commands.push(DrawCommand {
                texture_id: font.texture_id(),
                vertices: tv,
            });
        }
    }

    // ---------- Дерево ----------
    fn children(&self) -> &[Box<dyn RenderBox>] {
        &[]
    }

    fn children_mut(&mut self) -> &mut [Box<dyn RenderBox>] {
        &mut []
    }

    // ---------- Хит-тестирование ----------
    fn hit_test(&self, point: Point) -> bool {
        let rect = Rect::new(
            self.position.x,
            self.position.y,
            self.size.width,
            self.size.height,
        );
        rect.contains(point)
    }

    // ---------- Обработка событий ----------
    fn handle_event(&mut self, event: &Event, _ui_manager: &mut UiManager) -> bool {
        match event {
            Event::Click(_) => {
                if let Some(cb) = &mut self.on_click {
                    cb();
                }
                true
            }
            Event::PointerMove(_) => {
                self.is_hovered = true;
                true
            }
            Event::PointerDown(_) => {
                // Можно изменить состояние нажатия
                true
            }
            Event::PointerUp(_) => {
                // Сброс нажатия
                true
            }
            _ => false,
        }
    }

    // ---------- Фокус ----------
    fn can_focus(&self) -> bool {
        true
    }

    fn set_focused(&mut self, focused: bool) {
        self.is_focused = focused;
    }

    fn is_focused(&self) -> bool {
        self.is_focused
    }

    // ---------- Клавиатура ----------
    fn handle_key_down(&mut self, key: Key, _modifiers: KeyboardModifiers) -> bool {
        if key == Key::Enter || key == Key::CharInput(' ') {
            if let Some(cb) = &mut self.on_click {
                cb();
            }
            true
        } else {
            false
        }
    }

    fn handle_char_input(&mut self, ch: char) -> bool {
        if ch == ' ' {
            if let Some(cb) = &mut self.on_click {
                cb();
            }
            true
        } else {
            false
        }
    }

    // ---------- Drag & Drop (источник) ----------
    fn can_drag(&self) -> bool {
        true
    }

    fn drag_data(&self) -> Option<DragData> {
        Some(DragData::Text(self.text.clone()))
    }

    fn on_drag_start(&mut self, _point: Point) {
        // Можно изменить внешний вид при перетаскивании
        self.is_hovered = false;
    }

    fn on_drag_end(&mut self, _cancelled: bool) {
        // Восстановить внешний вид
    }

    // ---------- Drag & Drop (цель) ----------
    fn can_drop(&self, data: &DragData) -> bool {
        matches!(data, DragData::Text(_))
    }

    fn on_drag_enter(&mut self, data: &DragData, _point: Point) {
        if let DragData::Text(s) = data {
            eprintln!("Button drag enter with text: {}", s);
        }
        self.is_hovered = true;
    }

    fn on_drag_leave(&mut self) {
        self.is_hovered = false;
    }

    fn on_drop(&mut self, data: &DragData, _point: Point) {
        if let DragData::Text(s) = data {
            eprintln!("Button drop received text: {}", s);
            // Здесь можно обновить текст кнопки, если нужно:
            // self.text = s.clone();
        }
    }

    // ---------- Идентификатор ----------
    fn widget_id(&self) -> Option<WidgetId> {
        self.id
    }
}

// Реализация Drop (опционально)
impl Drop for ButtonRenderObject {
    fn drop(&mut self) {
        // Ничего особенного
    }
}