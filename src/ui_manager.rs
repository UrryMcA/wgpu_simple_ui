// src/ui/ui_manager.rs
use crate::common::event::{Event, KeyboardModifiers};
use crate::common::key::Key;
use crate::common::render_box::{RenderBox, WidgetId};
use crate::common::types::{BitmapFont, Constraints, LayoutContext, Point, Size};
use crate::common::primitives::Primitives;
use crate::common::vertex::DrawCommand;
use crate::common::render_context::RenderContext;
use crate::texture_manager::TextureManager;
use crate::font_system::FontSystem;
use crate::drag_drop_manager::DragDropManager;
use wgpu::Device;
use std::mem;

pub struct UiManager {
    root: Option<Box<dyn RenderBox>>,
    screen_size: Size,
    focused_widget_id: Option<WidgetId>,
    drag_drop: DragDropManager,
    font_system: FontSystem,
    texture_manager: TextureManager,
    primitives: Box<dyn Primitives>,
    scale_factor: f32,
}

impl UiManager {
    pub fn new(
        screen_size: Size,
        texture_manager: TextureManager,
        primitives: Box<dyn Primitives>,
        scale_factor: f32,
    ) -> Self {
        Self {
            root: None,
            screen_size,
            focused_widget_id: None,
            drag_drop: DragDropManager::new(),
            font_system: FontSystem::new(),
            texture_manager,
            primitives,
            scale_factor,
        }
    }

    pub fn set_root(&mut self, root: Box<dyn RenderBox>) {
        self.root = Some(root);
        self.layout(self.screen_size);
    }

    pub fn layout(&mut self, screen_size: Size) {
        self.screen_size = screen_size;
        if let Some(mut root) = self.root.take() {
            let constraints = Constraints::tight(self.screen_size.width, self.screen_size.height);
            root.layout(constraints, self);
            self.root = Some(root);
        }
    }

    pub fn render(&mut self, commands: &mut Vec<DrawCommand>, device: &Device) {
        if let Some(mut root) = self.root.take() {
            let mut ctx = RenderContext::new(
                commands,
                self.primitives.as_ref(),
                &mut self.texture_manager,
                &self.font_system,
                device,
            );
            root.render(&mut ctx);
            self.root = Some(root);
        }
    }

    // ---------- Управление шрифтами и текстурами ----------
    pub fn add_font(&mut self, name: String, font: Box<dyn BitmapFont>) {
        self.font_system.add_font(name, font);
    }

    pub fn font_system(&self) -> &FontSystem {
        &self.font_system
    }

    pub fn font_system_mut(&mut self) -> &mut FontSystem {
        &mut self.font_system
    }

    pub fn texture_manager(&self) -> &TextureManager {
        &self.texture_manager
    }

    pub fn texture_manager_mut(&mut self) -> &mut TextureManager {
        &mut self.texture_manager
    }

    pub fn get_font(&self, name: &str) -> Option<&dyn BitmapFont> {
        self.font_system.get_font(name)
    }

    // ---------- Доступ к виджетам по ID ----------
    pub fn with_widget_mut<F>(&mut self, id: WidgetId, mut f: F) -> bool
    where
        F: FnMut(&mut dyn RenderBox) -> bool,
    {
        if let Some(root) = &mut self.root {
            Self::with_widget_recursive(root.as_mut(), id, &mut f)
        } else {
            false
        }
    }

    fn with_widget_recursive<F>(node: &mut dyn RenderBox, id: WidgetId, f: &mut F) -> bool
    where
        F: FnMut(&mut dyn RenderBox) -> bool,
    {
        if node.widget_id() == Some(id) {
            return f(node);
        }
        for child in node.children_mut() {
            if Self::with_widget_recursive(child.as_mut(), id, f) {
                return true;
            }
        }
        false
    }

    pub fn send_event_to_widget(&mut self, id: WidgetId, event: &Event) -> bool {
        if let Some(mut root) = self.root.take() {
            let result = Self::send_event_recursive(root.as_mut(), id, event, self);
            self.root = Some(root);
            result
        } else {
            false
        }
    }

    fn send_event_recursive(
        node: &mut dyn RenderBox,
        id: WidgetId,
        event: &Event,
        ui: &mut UiManager,
    ) -> bool {
        if node.widget_id() == Some(id) {
            return node.handle_event(event, ui);
        }
        for child in node.children_mut() {
            if Self::send_event_recursive(child.as_mut(), id, event, ui) {
                return true;
            }
        }
        false
    }

    // ---------- Фокус ----------
    fn set_focus_to_widget(&mut self, id: WidgetId) -> bool {
        if self.focused_widget_id == Some(id) {
            return false;
        }
        if let Some(old_id) = self.focused_widget_id {
            self.with_widget_mut(old_id, |w| {
                w.set_focused(false);
                true
            });
        }
        let success = self.with_widget_mut(id, |w| {
            w.set_focused(true);
            true
        });
        if success {
            self.focused_widget_id = Some(id);
        } else {
            self.focused_widget_id = None;
        }
        success
    }

    fn clear_focus(&mut self) {
        if let Some(id) = self.focused_widget_id {
            self.with_widget_mut(id, |w| {
                w.set_focused(false);
                true
            });
            self.focused_widget_id = None;
        }
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
            Some(0) => all.len().saturating_sub(1),
            Some(idx) => idx - 1,
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
            self.with_widget_mut(id, |w| w.handle_key_down(key, modifiers))
        } else {
            false
        }
    }

    fn on_key_up(&mut self, key: Key, modifiers: KeyboardModifiers) -> bool {
        if let Some(id) = self.focused_widget_id {
            self.with_widget_mut(id, |w| w.handle_key_up(key, modifiers))
        } else {
            false
        }
    }

    fn on_char_input(&mut self, ch: char) -> bool {
        if let Some(id) = self.focused_widget_id {
            self.with_widget_mut(id, |w| w.handle_char_input(ch))
        } else {
            false
        }
    }

    // ---------- Хит-тестирование ----------
    pub fn hit_test(&self, point: Point) -> Option<WidgetId> {
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

    // ---------- Обработка событий ----------
    pub fn process_event(&mut self, event: &Event) -> bool {
        match event {
            Event::PointerDown(point) => {
                let mut drag_drop = mem::take(&mut self.drag_drop);
                drag_drop.on_pointer_down(*point, self);
                self.drag_drop = drag_drop;

                if !self.drag_drop.is_drag_active() {
                    if let Some(widget_id) = self.hit_test(*point) {
                        self.set_focus_to_widget(widget_id);
                    } else {
                        self.clear_focus();
                    }
                } else {
                    if let Some(widget_id) = self.hit_test(*point) {
                        self.send_event_to_widget(widget_id, &Event::PointerDown(*point));
                    }
                }
                false
            }
            Event::PointerUp(point) => {
                let mut drag_drop = mem::take(&mut self.drag_drop);
                let handled = drag_drop.on_pointer_up(*point, self);
                self.drag_drop = drag_drop;

                if handled {
                    return true;
                }
                if let Some(widget_id) = self.hit_test(*point) {
                    self.send_event_to_widget(widget_id, &Event::Click(*point));
                    return true;
                }
                false
            }
            Event::PointerMove(point) => {
                let mut drag_drop = mem::take(&mut self.drag_drop);
                let handled = drag_drop.on_pointer_move(*point, self);
                self.drag_drop = drag_drop;
                handled
            }
            Event::MouseWheel { delta_x, delta_y, point } => {
                if let Some(widget_id) = self.hit_test(*point) {
                    self.send_event_to_widget(widget_id, &Event::MouseWheel { delta_x: *delta_x, delta_y: *delta_y, point: *point });
                    true
                } else {
                    false
                }
            }
            Event::DragMove(point) => {
                let mut drag_drop = mem::take(&mut self.drag_drop);
                drag_drop.handle_drag_move(*point, self);
                self.drag_drop = drag_drop;
                true
            }
            Event::DragEnd { point, cancelled } => {
                let mut drag_drop = mem::take(&mut self.drag_drop);
                drag_drop.end_drag(*point, *cancelled, self);
                self.drag_drop = drag_drop;
                true
            }
            Event::KeyDown(key, mods) => self.on_key_down(*key, *mods),
            Event::KeyUp(key, mods) => self.on_key_up(*key, *mods),
            Event::CharInput(ch) => self.on_char_input(*ch),
            _ => false,
        }
    }
}

impl LayoutContext for UiManager {
    fn measure_text(&mut self, text: &str, font_size: f32, max_width: f32) -> Size {
        self.font_system.measure_text(text, font_size, max_width)
    }

    fn measure_text_with_font(&mut self, font_name: &str, text: &str, font_size: f32, max_width: f32) -> Size {
        if let Some(font) = self.font_system.get_font(font_name) {
            self.font_system.measure_text_with_font(font, text, font_size, max_width)
        } else {
            Size::zero()
        }
    }

    fn get_image_size(&mut self, path: &str) -> Option<Size> {
        self.texture_manager.get_size(path)
    }

    fn scale_factor(&self) -> f32 {
        self.scale_factor
    }
}

