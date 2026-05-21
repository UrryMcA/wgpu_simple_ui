// src/widgets/scrollable_container.rs
use crate::common::render_box::{RenderBox, WidgetId};
use crate::common::types::{Constraints, LayoutContext, Point, Rect, Size};
use crate::common::vertex::DrawCommand;
use crate::common::event::Event;
use crate::texture_manager::TextureManager;
use crate::ui::UiManager;

/// Контейнер с прокруткой. Оборачивает один дочерний виджет.
pub struct ScrollableContainer {
    child: Box<dyn RenderBox>,
    scroll_x: f32,
    scroll_y: f32,
    viewport_size: Size,
    id: Option<WidgetId>,
}

impl ScrollableContainer {
    pub fn new(child: Box<dyn RenderBox>) -> Self {
        Self {
            child,
            scroll_x: 0.0,
            scroll_y: 0.0,
            viewport_size: Size::default(),
            id: None,
        }
    }

    /// Установить размер области просмотра (обычно задаётся контейнером)
    pub fn set_viewport_size(&mut self, size: Size) {
        self.viewport_size = size;
    }

    /// Получить текущую прокрутку
    pub fn scroll(&self) -> (f32, f32) {
        (self.scroll_x, self.scroll_y)
    }

    /// Установить прокрутку (с ограничениями)
    pub fn set_scroll(&mut self, x: f32, y: f32) {
        let max_scroll_x = (self.child.size().width - self.viewport_size.width).max(0.0);
        let max_scroll_y = (self.child.size().height - self.viewport_size.height).max(0.0);
        self.scroll_x = x.clamp(0.0, max_scroll_x);
        self.scroll_y = y.clamp(0.0, max_scroll_y);
    }

    /// Прокрутить на дельту
    pub fn scroll_by(&mut self, dx: f32, dy: f32) {
        self.set_scroll(self.scroll_x + dx, self.scroll_y + dy);
    }
}

impl RenderBox for ScrollableContainer {
    fn layout(&mut self, constraints: Constraints, ctx: &mut dyn LayoutContext) -> Size {
        if self.id.is_none() {
            //self.id = Some(ui_manager.register_widget(self));
        }
        // Запоминаем размер области просмотра (ограничение)
        self.viewport_size = constraints.max;
        // Даём ребёнку бесконечные ограничения, чтобы он мог быть больше viewport
        let child_constraints = Constraints::loose();
        let child_size = self.child.layout(child_constraints, ctx);
        // Размер контейнера равен viewport (он не может быть больше)
        let size = constraints.constrain(self.viewport_size);
        self.viewport_size = size;
        size
    }

    fn set_position(&mut self, pos: Point) {
        // Позиция контейнера
        // Позиция ребёнка будет смещена на scroll при отрисовке и хит-тестировании
    }

    fn position(&self) -> Point {
        Point::default() // упрощённо
    }

    fn size(&self) -> Size {
        self.viewport_size
    }

    fn render(
        &self,
        commands: &mut Vec<DrawCommand>,
        primitives: &dyn crate::common::Primitives,
        textures: &TextureManager,
        ui_manager: &UiManager,
    ) {
        // Здесь нужно сместить отрисовку ребёнка на -scroll_x, -scroll_y
        // Для простоты пример не реализует clipping, но в реальности нужно обрезать.
        self.child.render(commands, primitives, textures, ui_manager);
    }

    fn children(&self) -> &[Box<dyn RenderBox>] {
        std::slice::from_ref(&self.child)
    }

    fn children_mut(&mut self) -> &mut [Box<dyn RenderBox>] {
        std::slice::from_mut(&mut self.child)
    }

    fn hit_test(&self, point: Point) -> bool {
        let rect = Rect::new(0.0, 0.0, self.viewport_size.width, self.viewport_size.height);
        rect.contains(point)
    }

    fn handle_event(&mut self, event: &Event, ui_manager: &mut UiManager) -> bool {
        // Сначала проверяем колёсико мыши
        if let Event::MouseWheel { delta_x, delta_y, point } = event {
            if self.hit_test(*point) {
                self.scroll_by(*delta_x, *delta_y);
                return true;
            }
        }
        // Для других событий нужно скорректировать точку с учётом прокрутки
        let mut event_clone = event.clone();
        if let Some(p) = event.point() {
            let adjusted = Point::new(p.x + self.scroll_x, p.y + self.scroll_y);
            // Меняем точку в событии (упрощённо, лучше создать новое событие)
            // Для полной реализации потребуется модификация Event или использование обходного пути.
        }
        // Передаём ребёнку (пропуская для краткости)
        false
    }

    fn handle_mouse_wheel(&mut self, delta_x: f32, delta_y: f32, point: Point) -> bool {
        if self.hit_test(point) {
            self.scroll_by(delta_x, delta_y);
            true
        } else {
            false
        }
    }

    fn widget_id(&self) -> Option<WidgetId> {
        self.id
    }
}