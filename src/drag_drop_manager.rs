// src/drag_drop_manager.rs
use crate::common::event::{DragData, Event};
use crate::common::render_box::WidgetId;
use crate::common::types::Point;
use crate::ui_manager::UiManager;

pub struct DragDropManager {
    drag_active: bool,
    drag_source_id: Option<WidgetId>,
    drag_data: Option<DragData>,
    drag_start_point: Option<Point>,
    potential_drag_source: Option<WidgetId>,
    current_drop_target: Option<WidgetId>,
    drag_threshold: f32,
}

impl DragDropManager {
    pub fn new() -> Self {
        Self {
            drag_active: false,
            drag_source_id: None,
            drag_data: None,
            drag_start_point: None,
            potential_drag_source: None,
            current_drop_target: None,
            drag_threshold: 5.0,
        }
    }

    pub fn is_drag_active(&self) -> bool {
        self.drag_active
    }

    pub fn on_pointer_down(&mut self, point: Point, ui: &mut UiManager) {
        if let Some(widget_id) = ui.hit_test(point) {
            let mut can_drag = false;
            ui.with_widget_mut(widget_id, |widget| {
                can_drag = widget.can_drag();
                true
            });
            if can_drag {
                self.potential_drag_source = Some(widget_id);
                self.drag_start_point = Some(point);
            }
        }
    }

    pub fn on_pointer_up(&mut self, point: Point, ui: &mut UiManager) -> bool {
        if self.drag_active {
            self.end_drag(point, false, ui);
            return true;
        }
        self.potential_drag_source = None;
        self.drag_start_point = None;
        false
    }

    pub fn on_pointer_move(&mut self, point: Point, ui: &mut UiManager) -> bool {
        if self.drag_active {
            self.handle_drag_move(point, ui);
            return true;
        }
        if let Some(source_id) = self.potential_drag_source {
            if let Some(start) = self.drag_start_point {
                let dx = point.x - start.x;
                let dy = point.y - start.y;
                if dx.hypot(dy) > self.drag_threshold {
                    self.start_drag(source_id, start, ui);
                    return true;
                }
            }
        }
        false
    }

    pub fn handle_drag_move(&mut self, point: Point, ui: &mut UiManager) {
        let new_target = ui.hit_test(point);
        if new_target != self.current_drop_target {
            if let Some(old) = self.current_drop_target {
                ui.send_event_to_widget(old, &Event::DragLeave);
            }
            if let Some(new) = new_target {
                if let Some(data) = &self.drag_data {
                    let mut can_drop = false;
                    ui.with_widget_mut(new, |widget| {
                        can_drop = widget.can_drop(data);
                        true
                    });
                    if can_drop {
                        ui.send_event_to_widget(new, &Event::DragEnter {
                            point,
                            data: data.clone(),
                        });
                        self.current_drop_target = Some(new);
                    } else {
                        self.current_drop_target = None;
                    }
                } else {
                    self.current_drop_target = None;
                }
            } else {
                self.current_drop_target = None;
            }
        }
        if let Some(source) = self.drag_source_id {
            ui.send_event_to_widget(source, &Event::DragMove(point));
        }
    }

    pub fn end_drag(&mut self, point: Point, cancelled: bool, ui: &mut UiManager) {
        if !cancelled {
            if let Some(target) = self.current_drop_target {
                if let Some(data) = &self.drag_data {
                    let mut can_drop = false;
                    ui.with_widget_mut(target, |widget| {
                        can_drop = widget.can_drop(data);
                        true
                    });
                    if can_drop {
                        ui.send_event_to_widget(target, &Event::DragDrop {
                            point,
                            data: data.clone(),
                        });
                    }
                }
            }
        }
        if let Some(source) = self.drag_source_id {
            ui.send_event_to_widget(source, &Event::DragEnd { point, cancelled });
        }
        self.drag_active = false;
        self.drag_source_id = None;
        self.drag_data = None;
        self.current_drop_target = None;
    }

    fn start_drag(&mut self, source_id: WidgetId, start_point: Point, ui: &mut UiManager) {
        let mut data = None;
        ui.with_widget_mut(source_id, |widget| {
            data = widget.drag_data();
            true
        });
        if let Some(data) = data {
            self.drag_active = true;
            self.drag_source_id = Some(source_id);
            self.drag_data = Some(data.clone());
            ui.send_event_to_widget(source_id, &Event::DragStart {
                point: start_point,
                source_id,
                data,
            });
        }
    }
}

impl Default for DragDropManager {
    fn default() -> Self {
        Self::new()
    }
}