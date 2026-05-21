// src/widgets/panel.rs
use super::widget::Widget;
use crate::common::render_box::RenderBox;
use crate::common::{DrawCommand, Primitives, types::*};
use crate::common::event::{Event, KeyboardModifiers, DragData};
use crate::common::key::Key;
use crate::texture_manager::TextureManager;
use crate::ui::UiManager;

/// Панель — контейнер с фоном, скруглёнными углами и отступами.
/// Может содержать любого одного дочернего виджета (или несколько, если использовать контейнер).
pub struct Panel {
    child: Box<dyn Widget>,
    background_color: UColor,
    corner_radius: f32,
    margin: EdgeInsets,
    padding: EdgeInsets,
}

impl Panel {
    pub fn new(child: Box<dyn Widget>) -> Self {
        Self {
            child,
            background_color: UColor([0.2, 0.2, 0.2, 1.0]), // тёмный фон по умолчанию
            corner_radius: 8.0,
            margin: EdgeInsets::default(),
            padding: EdgeInsets::all(12.0),
        }
    }

    pub fn background(mut self, color: UColor) -> Self {
        self.background_color = color;
        self
    }

    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }

    pub fn margin(mut self, margin: EdgeInsets) -> Self {
        self.margin = margin;
        self
    }

    pub fn padding(mut self, padding: EdgeInsets) -> Self {
        self.padding = padding;
        self
    }
}

impl Widget for Panel {
    fn min_size(&self) -> Size {
        // Минимальный размер = минимальный размер ребёнка + паддинги
        let child_min = self.child.min_size();
        let padded = self.padding.inflate(child_min);
        self.margin.inflate(padded)
    }

    fn margin(&self) -> EdgeInsets {
        self.margin
    }

    fn padding(&self) -> EdgeInsets {
        self.padding
    }

    fn create_render_object(&self) -> Box<dyn RenderBox> {
        let child_render = self.child.create_render_object();
        Box::new(PanelRenderObject {
            child: child_render,
            background_color: self.background_color,
            corner_radius: self.corner_radius,
            padding: self.padding,
            margin: self.margin,
            position: Point::default(),
            size: Size::default(),
            id: None,
        })
    }
}

/// Объект рендеринга панели
struct PanelRenderObject {
    child: Box<dyn RenderBox>,
    background_color: UColor,
    corner_radius: f32,
    padding: EdgeInsets,
    margin: EdgeInsets,
    position: Point,
    size: Size,
    id: Option<u64>, // WidgetId
}

impl PanelRenderObject {
    /// Внутренняя область после применения отступов (padding)
    fn inner_rect(&self) -> Rect {
        Rect::new(
            self.position.x + self.padding.left + self.margin.left,
            self.position.y + self.padding.top + self.margin.top,
            self.size.width - self.padding.left - self.padding.right - self.margin.left - self.margin.right,
            self.size.height - self.padding.top - self.padding.bottom - self.margin.top - self.margin.bottom,
        )
    }
}

impl RenderBox for PanelRenderObject {
    // ---------- Обязательные методы ----------
    fn layout(&mut self, constraints: Constraints, ctx: &mut dyn LayoutContext) -> Size {
        if self.id.is_none() {
            //self.id = Some(ui_manager.register_widget(self));
        }

        // Учитываем margin и padding при расчёте доступного места для ребёнка
        let margin_total = EdgeInsets {
            left: self.margin.left + self.padding.left,
            right: self.margin.right + self.padding.right,
            top: self.margin.top + self.padding.top,
            bottom: self.margin.bottom + self.padding.bottom,
        };
        let inner_constraints = constraints.shrink(margin_total);
        let child_size = self.child.layout(inner_constraints, ctx);
        let total_size = margin_total.inflate(child_size);
        let constrained_size = constraints.constrain(total_size);
        self.size = constrained_size;

        // Устанавливаем позицию ребёнка
        let inner_origin = Point::new(
            self.position.x + self.margin.left + self.padding.left,
            self.position.y + self.margin.top + self.padding.top,
        );
        self.child.set_position(inner_origin);

        self.size
    }

    fn set_position(&mut self, pos: Point) {
        self.position = pos;
        // Позиция ребёнка обновится при следующем layout
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
        textures: &TextureManager,
        ui_manager: &UiManager,
    ) {
        // Рисуем фон панели
        let rect = Rect::new(
            self.position.x + self.margin.left,
            self.position.y + self.margin.top,
            self.size.width - self.margin.left - self.margin.right,
            self.size.height - self.margin.top - self.margin.bottom,
        );
        let bg = primitives.rounded_rect_vertices(rect, self.corner_radius, self.background_color);
        commands.push(DrawCommand { texture_id: 0, vertices: bg });

        // Рендерим ребёнка
        self.child.render(commands, primitives, textures, ui_manager);
    }

    // ---------- Дерево ----------
    fn children(&self) -> &[Box<dyn RenderBox>] {
        std::slice::from_ref(&self.child)
    }

    fn children_mut(&mut self) -> &mut [Box<dyn RenderBox>] {
        std::slice::from_mut(&mut self.child)
    }

    // ---------- Хит-тестирование ----------
    fn hit_test(&self, point: Point) -> bool {
        let rect = Rect::new(
            self.position.x + self.margin.left,
            self.position.y + self.margin.top,
            self.size.width - self.margin.left - self.margin.right,
            self.size.height - self.margin.top - self.margin.bottom,
        );
        rect.contains(point)
    }

    // ---------- Обработка событий (передача ребёнку) ----------
    fn handle_event(&mut self, event: &Event, ui_manager: &mut UiManager) -> bool {
        // Сначала проверяем, попадает ли точка в панель (для мышиных событий)
        if let Some(point) = event.point() {
            if !self.hit_test(point) {
                return false;
            }
        }
        // Передаём ребёнку
        self.child.handle_event(event, ui_manager)
    }

    // ---------- Фокус (делегируем ребёнку) ----------
    fn can_focus(&self) -> bool {
        self.child.can_focus()
    }

    fn set_focused(&mut self, focused: bool) {
        self.child.set_focused(focused);
    }

    fn is_focused(&self) -> bool {
        self.child.is_focused()
    }

    // ---------- Клавиатура (делегируем ребёнку) ----------
    fn handle_key_down(&mut self, key: Key, modifiers: KeyboardModifiers) -> bool {
        self.child.handle_key_down(key, modifiers)
    }

    fn handle_key_up(&mut self, key: Key, modifiers: KeyboardModifiers) -> bool {
        self.child.handle_key_up(key, modifiers)
    }

    fn handle_char_input(&mut self, ch: char) -> bool {
        self.child.handle_char_input(ch)
    }

    // ---------- Drag & Drop (обёртка) ----------
    fn can_drag(&self) -> bool {
        self.child.can_drag()
    }

    fn drag_data(&self) -> Option<DragData> {
        self.child.drag_data()
    }

    fn on_drag_start(&mut self, point: Point) {
        self.child.on_drag_start(point);
    }

    fn on_drag_move(&mut self, point: Point) {
        self.child.on_drag_move(point);
    }

    fn on_drag_end(&mut self, cancelled: bool) {
        self.child.on_drag_end(cancelled);
    }

    fn can_drop(&self, data: &DragData) -> bool {
        self.child.can_drop(data)
    }

    fn on_drag_enter(&mut self, data: &DragData, point: Point) {
        self.child.on_drag_enter(data, point);
    }

    fn on_drag_leave(&mut self) {
        self.child.on_drag_leave();
    }

    fn on_drop(&mut self, data: &DragData, point: Point) {
        self.child.on_drop(data, point);
    }

    // ---------- Идентификатор ----------
    fn widget_id(&self) -> Option<u64> {
        self.id
    }
}