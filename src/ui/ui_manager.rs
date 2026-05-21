// src/ui/ui_manager.rs
use crate::common::event::{DragData, Event, KeyboardModifiers};
use crate::common::key::Key;
use crate::common::render_box::{RenderBox, WidgetId};
use crate::common::types::{Constraints, LayoutContext, Point, Size};
use std::collections::HashMap;

pub struct UiManager {
    root: Option<Box<dyn RenderBox>>,
    screen_size: Size,
    // Фокус
    focused_widget_id: Option<WidgetId>,
    widgets: HashMap<WidgetId, *mut dyn RenderBox>,
    next_widget_id: WidgetId,
    // DnD
    drag_active: bool,
    drag_source_id: Option<WidgetId>,
    drag_data: Option<DragData>,
    drag_start_point: Option<Point>,
    potential_drag_source: Option<WidgetId>,
    current_drop_target: Option<WidgetId>,
    drag_threshold: f32,
}

impl UiManager {
    pub fn new(screen_size: Size) -> Self {
        Self {
            root: None,
            screen_size,
            focused_widget_id: None,
            widgets: HashMap::new(),
            next_widget_id: 1,
            drag_active: false,
            drag_source_id: None,
            drag_data: None,
            drag_start_point: None,
            potential_drag_source: None,
            current_drop_target: None,
            drag_threshold: 5.0,
        }
    }

    pub fn set_root(&mut self, root: Box<dyn RenderBox>) {
        self.root = Some(root);
    }

    pub fn layout(&mut self) {
        if let Some(root) = &mut self.root {
            let constraints = Constraints::tight(self.screen_size);
            root.layout(constraints, self);
        }
    }

    pub fn draw(&mut self) {
        if let Some(root) = &self.root {
            root.render(&mut Vec::new(), 
                    &crate::common::primitives::Primitives::default(),
                     &crate::texture_manager::TextureManager::new(), 
                     self);
        }
    }

    pub fn register_widget(&mut self, widget: &mut dyn RenderBox) -> WidgetId {
        let id = self.next_widget_id;
        self.next_widget_id += 1;
        self.widgets.insert(id, widget as *mut dyn RenderBox);
        id
    }

    pub fn process_event(&mut self, event: &Event) -> bool {
        match event {
            Event::PointerDown(point) => self.on_pointer_down(*point),
            Event::PointerUp(point) => self.on_pointer_up(*point),
            Event::PointerMove(point) => self.on_pointer_move(*point),
            Event::MouseWheel { delta_x, delta_y, point } => self.on_mouse_wheel(*delta_x, *delta_y, *point),
            Event::KeyDown(key, mods) => self.on_key_down(*key, *mods),
            Event::KeyUp(key, mods) => self.on_key_up(*key, *mods),
            Event::CharInput(ch) => self.on_char_input(*ch),
            Event::DragMove(point) => self.handle_drag_move(*point),
            Event::DragEnd { point, cancelled } => self.end_drag(*point, *cancelled),
            _ => false,
        }
    }

    // ---------- Приватные обработчики ----------
    fn on_pointer_down(&mut self, point: Point) {
        if let Some(widget_id) = self.hit_test(point) {
            if let Some(widget) = self.get_widget_mut(widget_id) {
                if widget.can_drag() {
                    self.potential_drag_source = Some(widget_id);
                    self.drag_start_point = Some(point);
                }
            }
            self.set_focus_to_widget(widget_id);
        } else {
            self.clear_focus();
        }
    }

    fn on_pointer_up(&mut self, point: Point) {
        if self.drag_active {
            self.end_drag(point, false);
        } else {
            if let Some(widget_id) = self.hit_test(point) {
                self.send_event_to_widget(widget_id, &Event::Click(point));
            }
        }
        self.potential_drag_source = None;
        self.drag_start_point = None;
    }

    fn on_pointer_move(&mut self, point: Point) {
        if self.drag_active {
            self.handle_drag_move(point);
        } else if let Some(source_id) = self.potential_drag_source {
            if let Some(start) = self.drag_start_point {
                let dx = point.x - start.x;
                let dy = point.y - start.y;
                if dx.hypot(dy) > self.drag_threshold {
                    self.start_drag(source_id, start);
                }
            }
        }
    }

    fn on_mouse_wheel(&mut self, delta_x: f32, delta_y: f32, point: Point) {
        // Находим виджет под курсором и отправляем событие
        if let Some(widget_id) = self.hit_test(point) {
            self.send_event_to_widget(widget_id, &Event::MouseWheel { delta_x, delta_y, point });
        }
    }

    fn start_drag(&mut self, source_id: WidgetId, start_point: Point) {
        let data = self.get_widget_mut(source_id).and_then(|w| w.drag_data());
        if let Some(data) = data {
            self.drag_active = true;
            self.drag_source_id = Some(source_id);
            self.drag_data = Some(data.clone());
            self.send_event_to_widget(source_id, &Event::DragStart {
                point: start_point,
                source_id,
                data,
            });
        }
    }

    fn handle_drag_move(&mut self, point: Point) {
        let new_target = self.hit_test_drop_target(point);
        if new_target != self.current_drop_target {
            if let Some(old) = self.current_drop_target {
                self.send_event_to_widget(old, &Event::DragLeave);
            }
            if let Some(new) = new_target {
                if let Some(data) = &self.drag_data {
                    self.send_event_to_widget(new, &Event::DragEnter {
                        point,
                        data: data.clone(),
                    });
                }
            }
            self.current_drop_target = new_target;
        }
        if let Some(source) = self.drag_source_id {
            self.send_event_to_widget(source, &Event::DragMove(point));
        }
    }

    fn end_drag(&mut self, point: Point, cancelled: bool) {
        if !cancelled {
            if let Some(target) = self.current_drop_target {
                if let Some(data) = &self.drag_data {
                    self.send_event_to_widget(target, &Event::DragDrop {
                        point,
                        data: data.clone(),
                    });
                }
            }
        }
        if let Some(source) = self.drag_source_id {
            self.send_event_to_widget(source, &Event::DragEnd { point, cancelled });
        }
        self.drag_active = false;
        self.drag_source_id = None;
        self.drag_data = None;
        self.current_drop_target = None;
    }

    // ---------- Фокус ----------
    fn set_focus_to_widget(&mut self, id: WidgetId) -> bool {
        if self.focused_widget_id == Some(id) {
            return false;
        }
        if let Some(old_id) = self.focused_widget_id {
            if let Some(old) = self.get_widget_mut(old_id) {
                old.set_focused(false);
            }
        }
        if let Some(new) = self.get_widget_mut(id) {
            new.set_focused(true);
            self.focused_widget_id = Some(id);
            true
        } else {
            false
        }
    }

    fn clear_focus(&mut self) {
        if let Some(id) = self.focused_widget_id {
            if let Some(w) = self.get_widget_mut(id) {
                w.set_focused(false);
            }
        }
        self.focused_widget_id = None;
    }

    fn focus_next(&mut self) {
        let all = self.collect_focusable();
        let current_idx = self.focused_widget_id.and_then(|id| all.iter().position(|&i| i == id));
        let next_idx = match current_idx {
            Some(idx) if idx + 1 < all.len() => idx + 1,
            Some(_) => 0,
            None => 0,
        };
        if let Some(&next_id) = all.get(next_idx) {
            self.set_focus_to_widget(next_id);
        }
    }

    fn focus_previous(&mut self) {
        let all = self.collect_focusable();
        let current_idx = self.focused_widget_id.and_then(|id| all.iter().position(|&i| i == id));
        let prev_idx = match current_idx {
            Some(idx) if idx > 0 => idx - 1,
            Some(0) => all.len().saturating_sub(1),
            None => 0,
        };
        if let Some(&prev_id) = all.get(prev_idx) {
            self.set_focus_to_widget(prev_id);
        }
    }

    fn collect_focusable(&self) -> Vec<WidgetId> {
        let mut out = Vec::new();
        if let Some(root) = &self.root {
            Self::collect_focusable_from(root.as_ref(), &mut out);
        }
        out
    }

    fn collect_focusable_from(node: &dyn RenderBox, out: &mut Vec<WidgetId>) {
        if node.can_focus() {
            if let Some(id) = node.widget_id() {
                out.push(id);
            }
        }
        for child in node.children() {
            Self::collect_focusable_from(child.as_ref(), out);
        }
    }

    fn on_key_down(&mut self, key: Key, modifiers: KeyboardModifiers) -> bool {
        if key == Key::Tab {
            if modifiers.shift() {
                self.focus_previous();
            } else {
                self.focus_next();
            }
            return true;
        }
        if let Some(id) = self.focused_widget_id {
            if let Some(w) = self.get_widget_mut(id) {
                if w.handle_key_down(key, modifiers) {
                    return true;
                }
            }
        }
        false
    }

    fn on_key_up(&mut self, key: Key, modifiers: KeyboardModifiers) -> bool {
        if let Some(id) = self.focused_widget_id {
            if let Some(w) = self.get_widget_mut(id) {
                if w.handle_key_up(key, modifiers) {
                    return true;
                }
            }
        }
        false
    }

    fn on_char_input(&mut self, ch: char) -> bool {
        if let Some(id) = self.focused_widget_id {
            if let Some(w) = self.get_widget_mut(id) {
                if w.handle_char_input(ch) {
                    return true;
                }
            }
        }
        false
    }

    // ---------- Хит-тестирование ----------
    fn hit_test(&self, point: Point) -> Option<WidgetId> {
        self.root.as_ref().and_then(|root| self.hit_test_node(root.as_ref(), point))
    }

    fn hit_test_node(&self, node: &dyn RenderBox, point: Point) -> Option<WidgetId> {
        if !node.hit_test(point) {
            return None;
        }
        for child in node.children().iter().rev() {
            if let Some(id) = self.hit_test_node(child.as_ref(), point) {
                return Some(id);
            }
        }
        node.widget_id()
    }

    fn hit_test_drop_target(&self, point: Point) -> Option<WidgetId> {
        self.root.as_ref().and_then(|root| self.find_drop_target_node(root.as_ref(), point))
    }

    fn find_drop_target_node(&self, node: &dyn RenderBox, point: Point) -> Option<WidgetId> {
        if !node.hit_test(point) {
            return None;
        }
        for child in node.children().iter().rev() {
            if let Some(id) = self.find_drop_target_node(child.as_ref(), point) {
                return Some(id);
            }
        }
        if let Some(data) = &self.drag_data {
            if node.can_drop(data) {
                return node.widget_id();
            }
        }
        None
    }

    // ---------- Вспомогательные ----------
    fn get_widget_mut(&mut self, id: WidgetId) -> Option<&mut dyn RenderBox> {
        self.widgets.get(&id).copied().and_then(|ptr| unsafe { ptr.as_mut() })
    }

    fn send_event_to_widget(&mut self, id: WidgetId, event: &Event) -> bool {
        if let Some(w) = self.get_widget_mut(id) {
            w.handle_event(event, self)
        } else {
            false
        }
    }
}

impl LayoutContext for UiManager {
    fn measure_text(&mut self, text: &str, font_size: f32, max_width: f32) -> Size {
        self.font_system.measure(text, font_size, max_width)
    }
    fn get_image_size(&mut self, path: &str) -> Option<Size> {
        self.texture_manager.get_size(path)
    }
    fn scale_factor(&self) -> f32 {
        self.scale_factor
    }
}
