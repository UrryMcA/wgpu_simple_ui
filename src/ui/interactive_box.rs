use crate::common::event::{DragData, Event};
use crate::common::render_box::{RenderBox, WidgetId};
use crate::common::render_context::RenderContext;
use crate::common::types::{Constraints, EdgeInsets, LayoutContext, Point, Rect, Size};
use crate::ui::interactive_state::InteractiveState;
use crate::ui_manager::UiManager;
use crate::widgets::Widget;

/// Виджет-обёртка, добавляющий интерактивность (hover, click, drag, focus) любому другому виджету.
pub struct InteractiveBox<W: Widget> {
    child: W,
    state: InteractiveState,
    on_state_change: Option<Box<dyn FnMut(&InteractiveState) + Send>>,
}

impl<W: Widget> InteractiveBox<W> {
    pub fn new(child: W) -> Self {
        Self {
            child,
            state: InteractiveState::new(),
            on_state_change: None,
        }
    }

    pub fn on_click(mut self, f: impl FnMut() + Send + 'static) -> Self {
        self.state.set_on_click(f);
        self
    }

    pub fn drag_data(mut self, data: DragData) -> Self {
        self.state.set_drag_data(data);
        self
    }

    pub fn on_state_change(mut self, f: impl FnMut(&InteractiveState) + Send + 'static) -> Self {
        self.on_state_change = Some(Box::new(f));
        self
    }
    
    pub fn can_drop(mut self, f: impl Fn(&DragData) -> bool + Send + 'static) -> Self {
        self.state.set_can_drop(f);
        self
    }

    pub fn on_drop(mut self, f: impl Fn(DragData) + Send + 'static) -> Self {
        self.state.set_on_drop(f);
        self
    }    

}

impl<W: Widget> Widget for InteractiveBox<W> {
    fn min_size(&self, ctx: &mut dyn LayoutContext) -> Size {
        self.child.min_size(ctx)
    }

    fn margin(&self) -> EdgeInsets {
        self.child.margin()
    }

    fn padding(&self) -> EdgeInsets {
        self.child.padding()
    }

    fn create_render_object(&mut self) -> Box<dyn RenderBox> {
        let child_ro = self.child.create_render_object();
        let state = std::mem::take(&mut self.state);
        let on_state_change = self.on_state_change.take();

        Box::new(InteractiveBoxRenderObject {
            child: child_ro,
            state,
            on_state_change,
            position: Point::default(),
            size: Size::default(),
        })
    }
}

struct InteractiveBoxRenderObject {
    child: Box<dyn RenderBox>,
    state: InteractiveState,
    on_state_change: Option<Box<dyn FnMut(&InteractiveState) + Send>>,
    position: Point,
    size: Size,
}

impl InteractiveBoxRenderObject {
    fn notify_state_change(&mut self) {
        if let Some(cb) = &mut self.on_state_change {
            cb(&self.state);
        }
    }

    /// Синхронизирует состояние с дочерним виджетом и уведомляет об изменении.
    fn sync_state(&mut self) {
        self.child.update_interactive_state(&self.state);
        self.notify_state_change();
    }
}

impl RenderBox for InteractiveBoxRenderObject {
    fn layout(&mut self, constraints: Constraints, ctx: &mut dyn LayoutContext) -> Size {
        let new_size = self.child.layout(constraints, ctx);
        if new_size != self.size {
            self.size = new_size;
        }
        new_size
    }

    fn set_position(&mut self, pos: Point) {
        self.position = pos;
        self.child.set_position(pos);
    }

    fn position(&self) -> Point {
        self.position
    }

    fn size(&self) -> Size {
        self.size
    }

    fn render(&mut self, ctx: &mut RenderContext) {
        self.child.render(ctx);
    }

    fn children(&self) -> &[Box<dyn RenderBox>] {
        std::slice::from_ref(&self.child)
    }

    fn children_mut(&mut self) -> &mut [Box<dyn RenderBox>] {
        std::slice::from_mut(&mut self.child)
    }

    fn hit_test(&self, point: Point) -> bool {
        self.child.hit_test(point)
    }

    fn handle_event(&mut self, event: &Event, ui_manager: &mut UiManager) -> bool {
        let rect = Rect::new(self.position.x, self.position.y, self.size.width, self.size.height);
        println!("[BOX] rect={:?}", rect);
        let handled = self.state.handle_event(event, rect, ui_manager);
        // Всегда синхронизируем состояние, так как hovered мог измениться
        self.sync_state();
        if handled {
            return true;
        }
        self.child.handle_event(event, ui_manager)
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn set_focused(&mut self, focused: bool) {
        self.state.set_focused(focused);
        self.child.set_focused(focused);
        self.sync_state();
    }

    fn is_focused(&self) -> bool {
        self.state.focused
    }

    fn can_drag(&self) -> bool {
        self.state.drag_data.is_some()
    }

    fn drag_data(&self) -> Option<DragData> {
        self.state.drag_data.clone()
    }

    fn on_drag_start(&mut self, point: Point) {
        self.state.dragging = true;
        self.child.on_drag_start(point);
        self.sync_state();
    }

    fn on_drag_move(&mut self, point: Point) {
        self.child.on_drag_move(point);
    }

    fn on_drag_end(&mut self, cancelled: bool) {
        self.state.dragging = false;
        self.child.on_drag_end(cancelled);
        self.sync_state();
    }

    fn on_drag_enter(&mut self, data: &DragData, point: Point) {
        self.child.on_drag_enter(data, point);
    }

    fn on_drag_leave(&mut self) {
        self.child.on_drag_leave();
    }

    fn widget_id(&self) -> Option<WidgetId> {
        self.child.widget_id()
    }

    fn margin(&self) -> EdgeInsets {
        self.child.margin()
    }

    // Делегирование методов обновления состояния дочернему виджету
    fn update_interactive_state(&mut self, state: &InteractiveState) {
        self.child.update_interactive_state(state);
    }

    fn update_drag_state(&mut self, is_source: bool, is_target: bool) {
        self.child.update_drag_state(is_source, is_target);
    }

    fn set_widget_id(&mut self, id: WidgetId) {
        self.child.set_widget_id(id);
    }

    fn can_drop(&self, data: &DragData) -> bool {
        if let Some(check) = &self.state.can_drop_check {
            check(data)
        } else {
            self.child.can_drop(data)
        }
    }
    fn on_drop(&mut self, data: &DragData, point: Point) {
        if let Some(cb) = &mut self.state.on_drop_callback {
            cb(data.clone());
        } else {
            self.child.on_drop(data, point);
        }
    }

}