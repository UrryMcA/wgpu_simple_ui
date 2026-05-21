// src/widgets/scrollable_container.rs
use crate::common::render_box::{RenderBox, WidgetId};
use crate::common::types::{Constraints, EdgeInsets, LayoutContext, Point, Rect, Size};
use crate::common::vertex::DrawCommand;
use crate::common::event::Event;
use crate::common::primitives::Primitives;
use crate::texture_manager::TextureManager;
use crate::ui_manager::UiManager;

pub struct ScrollableContainer {
    child: Box<dyn RenderBox>,
    scroll_x: f32,
    scroll_y: f32,
    viewport_size: Size,
    margin: EdgeInsets,
    id: Option<WidgetId>,
}

impl ScrollableContainer {
    pub fn new(child: Box<dyn RenderBox>) -> Self {
        Self {
            child,
            scroll_x: 0.0,
            scroll_y: 0.0,
            viewport_size: Size::default(),
            margin: EdgeInsets::default(),
            id: None,
        }
    }
    pub fn margin(mut self, m: EdgeInsets) -> Self { self.margin = m; self }
    pub fn set_viewport_size(&mut self, size: Size) { self.viewport_size = size; }
    pub fn scroll(&self) -> (f32, f32) { (self.scroll_x, self.scroll_y) }
    pub fn set_scroll(&mut self, x: f32, y: f32) {
        let max_x = (self.child.size().width - self.viewport_size.width).max(0.0);
        let max_y = (self.child.size().height - self.viewport_size.height).max(0.0);
        self.scroll_x = x.clamp(0.0, max_x);
        self.scroll_y = y.clamp(0.0, max_y);
    }
    pub fn scroll_by(&mut self, dx: f32, dy: f32) { self.set_scroll(self.scroll_x + dx, self.scroll_y + dy); }
}

impl RenderBox for ScrollableContainer {
    fn layout(&mut self, constraints: Constraints, ctx: &mut dyn LayoutContext) -> Size {
        if self.id.is_none() { /* регистрация */ }
        let inner_constraints = Constraints {
            min_width: (constraints.min_width - self.margin.left - self.margin.right).max(0.0),
            max_width: (constraints.max_width - self.margin.left - self.margin.right).max(0.0),
            min_height: (constraints.min_height - self.margin.top - self.margin.bottom).max(0.0),
            max_height: (constraints.max_height - self.margin.top - self.margin.bottom).max(0.0),
        };
        self.viewport_size = constraints.constrain(Size::new(
            inner_constraints.max_width,
            inner_constraints.max_height,
        ));
        let child_constraints = Constraints::loose();
        let _child_size = self.child.layout(child_constraints, ctx);
        let total = Size::new(
            self.viewport_size.width + self.margin.left + self.margin.right,
            self.viewport_size.height + self.margin.top + self.margin.bottom,
        );
        constraints.constrain(total)
    }
    fn set_position(&mut self, _pos: Point) { /* позиция контейнера */ }
    fn position(&self) -> Point { Point::default() }
    fn size(&self) -> Size { self.viewport_size }

    fn render(&mut self, commands: &mut Vec<DrawCommand>, primitives: &dyn Primitives, textures: &TextureManager, ui_manager: &UiManager) {
        let old_pos = self.child.position();
        self.child.set_position(Point::new(old_pos.x - self.scroll_x, old_pos.y - self.scroll_y));
        self.child.render(commands, primitives, textures, ui_manager);
        self.child.set_position(old_pos);
    }

    fn children(&self) -> &[Box<dyn RenderBox>] { std::slice::from_ref(&self.child) }
    fn children_mut(&mut self) -> &mut [Box<dyn RenderBox>] { std::slice::from_mut(&mut self.child) }

    fn hit_test(&self, point: Point) -> bool {
        let rect = Rect::new(0.0, 0.0, self.viewport_size.width, self.viewport_size.height);
        rect.contains(point)
    }
    fn handle_event(&mut self, event: &Event, _ui_manager: &mut UiManager) -> bool {
        if let Event::MouseWheel { delta_x, delta_y, point } = event {
            if self.hit_test(*point) {
                self.scroll_by(*delta_x, *delta_y);
                return true;
            }
        }
        false
    }
    fn handle_mouse_wheel(&mut self, delta_x: f32, delta_y: f32, point: Point) -> bool {
        if self.hit_test(point) { self.scroll_by(delta_x, delta_y); true } else { false }
    }
    fn widget_id(&self) -> Option<WidgetId> { self.id }
    fn margin(&self) -> EdgeInsets { self.margin }
}