use crate::common::Primitives;
use crate::common::event::{DragData, Event, KeyboardModifiers};
use crate::common::key::Key;
use crate::common::types::{Constraints, EdgeInsets, LayoutContext, Point, Rect, Size};
use crate::common::vertex::DrawCommand;
use crate::texture_manager::TextureManager;
use crate::ui_manager::UiManager;
use std::any::Any;

/// Уникальный идентификатор виджета (для фокуса и хит-тестирования)
pub type WidgetId = u64;

/// Основной трейт для всех объектов рендеринга.
pub trait RenderBox: Any {
    // ---------- Обязательные методы ----------
    fn render(
        &mut self,
        commands: &mut Vec<DrawCommand>,
        primitives: &dyn Primitives,
        textures: &TextureManager,
        ui_manager: &mut UiManager,   // изменено: &mut вместо &
    );

    /// Вычисляет размер виджета в соответствии с constraints, используя контекст.
    /// После вызова layout, виджет должен запомнить свой final_size.
    fn layout(&mut self, constraints: Constraints, ctx: &mut dyn LayoutContext) -> Size;
    
    /// Установить позицию (вызывается контейнером после layout).
    fn set_position(&mut self, position: Point);
    
    /// Получить текущий размер (после layout).
    fn size(&self) -> Size;
    
    /// Получить позицию.
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
        let rect = Rect::new(
            self.position().x,
            self.position().y,
            self.size().width,
            self.size().height,
        );
        rect.contains(point)
    }

    // ---------- Обработка событий (базовая маршрутизация) ----------
    fn handle_event(&mut self, event: &Event, ui_manager: &mut UiManager) -> bool {
        // Если у события нет точки, пробуем отдать детям без проверки попадания?
        // Лучше проверять только если точка есть.
        if let Some(point) = event.point() {
            for child in self.children_mut().iter_mut().rev() {
                if child.hit_test(point) && child.handle_event(event, ui_manager) {
                    return true;
                }
            }
        } else {
            // События без точки (например, клавиатурные) отправляем всем детям? Обычно нет.
            // Для безопасности просто игнорируем или отправляем первому? Оставим false.
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

    // ---------- Колесо мыши (scroll) ----------
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

    // ---------- Идентификатор для системы фокуса ----------
    fn widget_id(&self) -> Option<WidgetId> {
        None
    }

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
