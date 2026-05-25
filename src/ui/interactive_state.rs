// src/ui/interactive_state.rs
use crate::common::event::{DragData, Event, KeyboardModifiers};
use crate::common::key::Key;
use crate::common::types::{Point, Rect};
use crate::ui_manager::UiManager;

/// Состояния интерактивного элемента (кнопки, перетаскиваемого виджета и т.д.)
pub struct InteractiveState {
    /// Наведён ли курсор на область виджета
    pub hovered: bool,
    /// Зажат ли левая кнопка мыши на виджете
    pub pressed: bool,
    /// Имеет ли виджет фокус клавиатуры
    pub focused: bool,
    /// Активно ли перетаскивание (drag) из этого виджета
    pub dragging: bool,
    /// Колбэк на клик (вызывается при отпускании кнопки, если нажатие было на виджете)
    pub on_click: Option<Box<dyn FnMut() + Send>>,
    /// Данные для перетаскивания (если виджет может быть источником drag)
    pub drag_data: Option<DragData>,
}

impl InteractiveState {
    pub fn new() -> Self {
        Self {
            hovered: false,
            pressed: false,
            focused: false,
            dragging: false,
            on_click: None,
            drag_data: None,
        }
    }

    /// Установить колбэк на клик
    pub fn set_on_click(&mut self, f: impl FnMut() + Send + 'static) {
        self.on_click = Some(Box::new(f));
    }

    /// Установить данные для перетаскивания (виджет может быть источником drag)
    pub fn set_drag_data(&mut self, data: DragData) {
        self.drag_data = Some(data);
    }

    /// Обработать событие, обновляя состояния и вызывая колбэки.
    /// Возвращает `true`, если событие было поглощено (обработано).
    /// `widget_rect` – область виджета в абсолютных координатах (используется для hit test).
    /// `ui_manager` – нужен для отправки событий drag & drop (может быть использован для вызова start_drag и т.д.)
    pub fn handle_event(
        &mut self,
        event: &Event,
        widget_rect: Rect,
        ui_manager: &mut UiManager,
    ) -> bool {
        match event {
            Event::PointerMove(point) => {
                let inside = widget_rect.contains(*point);
                if inside != self.hovered {
                    self.hovered = inside;
                }
                false
            }
            Event::PointerDown(point) => {
                if widget_rect.contains(*point) {
                    self.pressed = true;
                    true
                } else {
                    false
                }
            }
            Event::PointerUp(point) => {
                let was_pressed = self.pressed;
                self.pressed = false;
                if was_pressed && widget_rect.contains(*point) {
                    if let Some(cb) = &mut self.on_click {
                        cb();
                    }
                    true
                } else {
                    false
                }
            }
            Event::KeyDown(key, modifiers) => {
                if self.focused {
                    match key {
                        Key::Enter | Key::Space => {
                            if let Some(cb) = &mut self.on_click {
                                cb();
                            }
                            return true;
                        }
                        _ => {}
                    }
                }
                false
            }
            Event::KeyUp(..) => false,
            Event::CharInput(ch) => {
                if self.focused && (ch == &' ' || ch == &'\n') {
                    if let Some(cb) = &mut self.on_click {
                        cb();
                    }
                    true
                } else {
                    false
                }
            }
            Event::DragStart { .. } => {
                self.dragging = true;
                true
            }
            Event::DragEnd { cancelled: _, .. } => {
                self.dragging = false;
                true
            }
            _ => false,
        }
    }

    /// Сбросить все интерактивные состояния (например, при потере фокуса)
    pub fn reset(&mut self) {
        self.hovered = false;
        self.pressed = false;
        self.dragging = false;
    }

    /// Изменить фокус
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Вспомогательные методы для изменения цвета в зависимости от состояния
    pub fn modify_color(&self, normal: [f32; 4], pressed: [f32; 4], hovered: [f32; 4]) -> [f32; 4] {
        if self.dragging {
            pressed
        } else if self.pressed {
            pressed
        } else if self.hovered || self.focused {
            hovered
        } else {
            normal
        }
    }

    /// Модификация альфа-канала (например, для tint текстуры)
    pub fn modify_alpha(&self, normal_alpha: f32, pressed_alpha: f32, hovered_alpha: f32) -> f32 {
        if self.dragging {
            pressed_alpha
        } else if self.pressed {
            pressed_alpha
        } else if self.hovered || self.focused {
            hovered_alpha
        } else {
            normal_alpha
        }
    }
}

impl Default for InteractiveState {
    fn default() -> Self {
        Self::new()
    }
}