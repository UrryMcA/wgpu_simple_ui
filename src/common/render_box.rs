use crate::common::event::{DragData, Event, KeyboardModifiers};
use crate::common::key::Key;
use crate::common::render_context::RenderContext;
use crate::common::types::{Constraints, EdgeInsets, LayoutContext, Point, Rect, Size};
use crate::ui::interactive_state::InteractiveState;
use crate::ui_manager::UiManager;
use std::any::Any;

pub type WidgetId = u64;

pub trait RenderBox: Any {
    // ---------- Обязательные методы ----------
    fn render(&mut self, ctx: &mut RenderContext);
    fn layout(&mut self, constraints: Constraints, ctx: &mut dyn LayoutContext) -> Size;
    fn set_position(&mut self, position: Point);
    fn size(&self) -> Size;
    fn position(&self) -> Point;

    // ---------- Методы для работы с деревом ----------
    fn children(&self) -> &[Box<dyn RenderBox>] {
        &[]
    }
    fn children_mut(&mut self) -> &mut [Box<dyn RenderBox>] {
        &mut []
    }

    // ---------- Хит-тестирование ----------
    fn hit_test(&self, point: Point) -> bool {
        Rect::new(
            self.position().x,
            self.position().y,
            self.size().width,
            self.size().height,
        ).contains(point)
    }

    // ---------- Обработка событий ----------
    fn handle_event(&mut self, event: &Event, ui_manager: &mut UiManager) -> bool {
        if let Some(point) = event.point() {
            for child in self.children_mut().iter_mut().rev() {
                if child.hit_test(point) && child.handle_event(event, ui_manager) {
                    return true;
                }
            }
        }
        false
    }

    // ---------- Фокус ----------
    fn can_focus(&self) -> bool {
        false
    }
    fn set_focused(&mut self, _focused: bool) {}
    fn is_focused(&self) -> bool {
        false
    }

    // ---------- Клавиатура ----------
    fn handle_key_down(&mut self, _key: Key, _modifiers: KeyboardModifiers) -> bool {
        false
    }
    fn handle_key_up(&mut self, _key: Key, _modifiers: KeyboardModifiers) -> bool {
        false
    }
    fn handle_char_input(&mut self, _ch: char) -> bool {
        false
    }

    // ---------- Колесо мыши ----------
    fn handle_mouse_wheel(&mut self, _delta_x: f32, _delta_y: f32, _point: Point) -> bool {
        false
    }

    // ---------- Drag & Drop (источник) ----------
    fn can_drag(&self) -> bool {
        false
    }
    fn drag_data(&self) -> Option<DragData> {
        None
    }
    fn on_drag_start(&mut self, _point: Point) {}
    fn on_drag_move(&mut self, _point: Point) {}
    fn on_drag_end(&mut self, _cancelled: bool) {}

    // ---------- Drag & Drop (цель) ----------
    fn can_drop(&self, _data: &DragData) -> bool {
        false
    }
    fn on_drag_enter(&mut self, _data: &DragData, _point: Point) {}
    fn on_drag_leave(&mut self) {}
    fn on_drop(&mut self, _data: &DragData, _point: Point) {}

    // ---------- Идентификатор ----------
    fn widget_id(&self) -> Option<WidgetId> {
        None
    }

    // ---------- НОВЫЕ МЕТОДЫ (пункт 1 плана) ----------
    /// Обновляет интерактивное состояние виджета (наведение, нажатие, фокус, перетаскивание).
    /// По умолчанию ничего не делает. Виджеты, которые хотят реагировать на состояния, переопределяют этот метод.
    fn update_interactive_state(&mut self, _state: &InteractiveState) {}

    /// Обновляет состояние, связанное с drag & drop (является ли виджет источником или целью).
    /// По умолчанию ничего не делает.
    fn update_drag_state(&mut self, _is_source: bool, _is_target: bool) {}

    /// Устанавливает уникальный идентификатор виджета.
    /// По умолчанию ничего не делает. Виджеты, поддерживающие ID, переопределяют этот метод.
    fn set_widget_id(&mut self, _id: WidgetId) {}

    // ---------- Вспомогательные ----------
    fn rect(&self) -> Rect {
        Rect::new(
            self.position().x,
            self.position().y,
            self.size().width,
            self.size().height,
        )
    }
    fn margin(&self) -> EdgeInsets {
        EdgeInsets::default()
    }
}