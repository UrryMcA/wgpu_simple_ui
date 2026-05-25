// src/ui/interactive_box.rs
use crate::common::event::{DragData, Event};
use crate::common::render_box::{RenderBox, WidgetId};
use crate::common::render_context::RenderContext;
use crate::common::types::{Constraints, EdgeInsets, LayoutContext, Point, Rect, Size};
use crate::ui::interactive_state::InteractiveState;
use crate::ui_manager::UiManager;
use crate::widgets::Widget;

/// Виджет-обёртка, добавляющий интерактивность (hover, click, drag, focus) любому другому виджету.
/// Не изменяет внешний вид автоматически, но предоставляет состояние, которое можно использовать
/// в колбэке `on_state_change` для модификации отрисовки (например, изменить цвет фона).
pub struct InteractiveBox<W: Widget> {
    child: W,
    state: InteractiveState,
    /// Опциональный колбэк, вызываемый при изменении состояния (hover, press, focus, drag).
    /// Позволяет динамически менять внешний вид обёрнутого виджета.
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
        let mut state = std::mem::take(&mut self.state);
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

    // Вызывается при изменении любого состояния, чтобы можно было перерисовать обёрнутый виджет
    fn mark_child_dirty(&mut self) {
        // Простейший способ: если у child есть механизм dirty, можно вызвать его,
        // но у RenderBox нет такого метода. Поэтому полагаемся на то, что child перерисуется,
        // когда его попросят. Здесь мы ничего не делаем, т.к. перерисовка произойдёт
        // при следующем render().
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
        // Перед рендером можно применить состояние к child, если нужно,
        // но так как у нас нет доступа к его внутренностям, просто рисуем.
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
        let handled = self.state.handle_event(event, rect, ui_manager);
        if handled {
            // Состояние изменилось – уведомляем
            self.notify_state_change();
            // Для событий, которые изменили состояние (например, PointerDown), мы уже обработали,
            // и не передаём ребёнку? Обычно кнопка поглощает PointerDown, чтобы не мешать другим.
            // Возвращаем true.
            return true;
        }
        // Если событие не было обработано состоянием, передаём ребёнку (например, клавиатура, если фокус на ребёнке)
        self.child.handle_event(event, ui_manager)
    }

    fn can_focus(&self) -> bool {
        true // Интерактивный виджет может получать фокус
    }

    fn set_focused(&mut self, focused: bool) {
        self.state.set_focused(focused);
        self.child.set_focused(focused);
        self.notify_state_change();
    }

    fn is_focused(&self) -> bool {
        self.state.focused
    }

    // Drag & Drop – делегируем состоянию, а также источнику (ребёнку)
    fn can_drag(&self) -> bool {
        self.state.drag_data.is_some()
    }

    fn drag_data(&self) -> Option<DragData> {
        self.state.drag_data.clone()
    }

    fn on_drag_start(&mut self, point: Point) {
        self.state.dragging = true;
        self.child.on_drag_start(point);
        self.notify_state_change();
    }

    fn on_drag_move(&mut self, point: Point) {
        self.child.on_drag_move(point);
    }

    fn on_drag_end(&mut self, cancelled: bool) {
        self.state.dragging = false;
        self.child.on_drag_end(cancelled);
        self.notify_state_change();
    }

    fn can_drop(&self, data: &DragData) -> bool {
        self.child.can_drop(data)
    }

    fn on_drag_enter(&mut self, data: &DragData, point: Point) {
        self.child.on_drag_enter(data, point);
        // Можно изменить состояние на hovered? Но это уже делается в PointerMove.
    }

    fn on_drag_leave(&mut self) {
        self.child.on_drag_leave();
    }

    fn on_drop(&mut self, data: &DragData, point: Point) {
        self.child.on_drop(data, point);
    }

    fn widget_id(&self) -> Option<WidgetId> {
        self.child.widget_id()
    }

    fn margin(&self) -> EdgeInsets {
        self.child.margin()
    }
}
