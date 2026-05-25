/*

use crate::common::render_box::{RenderBox, WidgetId};
use crate::common::render_context::RenderContext;
use crate::common::types::*;
use crate::common::vertex::Vertex;
use crate::common::Primitives;
use crate::common::event::{Event, KeyboardModifiers};
use crate::common::key::Key;
use crate::ui_manager::UiManager;
use crate::ui::canvas::CanvasItem;
use crate::ui::widget::{Widget, LeafRenderObjectWidget};

/// Кнопка с кастомным фоном (Canvas) и произвольным контентом (Layout)
pub struct CanvasButton {
    desired_width: f32,
    desired_height: f32,
    items: Vec<CanvasItem>,
    content: Option<Box<dyn Widget>>,
    margin: EdgeInsets,
    on_click: Option<Box<dyn Fn() + Send + Sync>>,
}

impl CanvasButton {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            desired_width: width,
            desired_height: height,
            items: Vec::new(),
            content: None,
            margin: EdgeInsets::default(),
            on_click: None,
        }
    }

    pub fn add_canvas_item(mut self, item: CanvasItem) -> Self {
        self.items.push(item);
        self
    }

    /// Принимает ЛЮБОЙ виджет (Container, Stack, Panel и т.д.)
    pub fn content(mut self, widget: impl Widget + 'static) -> Self {
        self.content = Some(Box::new(widget));
        self
    }

    pub fn on_click(mut self, f: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_click = Some(Box::new(f));
        self
    }

    pub fn margin(mut self, m: EdgeInsets) -> Self {
        self.margin = m;
        self
    }
}

impl Widget for CanvasButton {
    fn min_size(&self, _ctx: &mut dyn LayoutContext) -> Size {
        Size::new(self.desired_width, self.desired_height)
    }

    fn margin(&self) -> EdgeInsets { self.margin }
    fn padding(&self) -> EdgeInsets { EdgeInsets::default() }

    fn create_render_object(&mut self) -> Box<dyn RenderBox> {
        let content_render = self.content.as_mut().map(|c| c.create_render_object());
        
        Box::new(CanvasButtonRenderObject {
            items: std::mem::take(&mut self.items),
            content: content_render,
            desired_width: self.desired_width,
            desired_height: self.desired_height,
            position: Point::default(),
            size: Size::default(),
            on_click: self.on_click.take(),
            is_hovered: false,
            cached_vertices: Vec::new(),
            cached_indices: Vec::new(),
            dirty: true,
        })
    }
}

impl LeafRenderObjectWidget for CanvasButton {}

struct CanvasButtonRenderObject {
    items: Vec<CanvasItem>,
    content: Option<Box<dyn RenderBox>>,
    desired_width: f32,
    desired_height: f32,
    position: Point,
    size: Size,
    on_click: Option<Box<dyn Fn() + Send + Sync>>,
    is_hovered: bool,
    cached_vertices: Vec<Vertex>,
    cached_indices: Vec<u32>,
    dirty: bool,
}

impl CanvasButtonRenderObject {
    fn rebuild_cache(&mut self, primitives: &dyn Primitives) {
        if self.size.width < 1.0 || self.size.height < 1.0 { return; }

        self.cached_vertices.clear();
        self.cached_indices.clear();

        // Рисуем фон (CanvasItems) в локальных координатах (0,0 - верхний левый угол кнопки)
        for item in &self.items {
            let (verts, inds) = match item {
                CanvasItem::Rect { rect, color } => primitives.rect_vertices_indices(*rect, *color),
                CanvasItem::RoundedRect { rect, radius, color } => primitives.rounded_rect_vertices_indices(*rect, *radius, *color),
                CanvasItem::OutlineRect { rect, radius, thickness, color } => primitives.rounded_rect_outline_vertices_indices(*rect, *radius, *thickness, *color),
                CanvasItem::Line { line, color } => primitives.line_vertices_indices(*line, *color),
                CanvasItem::Arc { arc, color } => primitives.arc_vertices_indices(*arc, *color),
                CanvasItem::FilledSector { sector, color } => primitives.filled_arc_sector_vertices_indices(*sector, *color),
                CanvasItem::Custom(f) => f(primitives),
            };
            let base = self.cached_vertices.len() as u32;
            self.cached_vertices.extend(verts);
            self.cached_indices.extend(inds.into_iter().map(|i| i + base));
        }
        self.dirty = false;
    }

    fn update_content_position(&mut self) {
        if let Some(ref mut content) = self.content {
            let content_size = content.size();
            // Центрируем контент относительно ПОЗИЦИИ КНОПКИ
            let pos_x = self.position.x + (self.size.width - content_size.width) / 2.0;
            let pos_y = self.position.y + (self.size.height - content_size.height) / 2.0;
            content.set_position(Point::new(pos_x, pos_y));
        }
    }
}

impl RenderBox for CanvasButtonRenderObject {
    fn layout(&mut self, constraints: Constraints, ctx: &mut dyn LayoutContext) -> Size {
        let desired = Size::new(self.desired_width, self.desired_height);
        let final_size = constraints.constrain(desired);

        if (final_size.width - self.size.width).abs() > 0.01 || (final_size.height - self.size.height).abs() > 0.01 {
            self.size = final_size;
            self.dirty = true;
        }

        // Делаем layout контента, ограничивая его размерами кнопки
        if let Some(ref mut content) = self.content {
            let content_constraints = Constraints {
                min_width: 0.0,
                max_width: self.size.width,
                min_height: 0.0,
                max_height: self.size.height,
            };
            content.layout(content_constraints, ctx);
            self.update_content_position();
        }

        final_size
    }

    fn set_position(&mut self, pos: Point) {
        if self.position != pos {
            self.position = pos;
            self.dirty = true;
            self.update_content_position();
        }
    }

    fn position(&self) -> Point { self.position }
    fn size(&self) -> Size { self.size }

    fn render(&mut self, ctx: &mut RenderContext) {
        if self.dirty { self.rebuild_cache(ctx.primitives); }

        // 1. Рендер фона (сдвигаем вершины на глобальную позицию кнопки)
        if !self.cached_vertices.is_empty() {
            let mut world_verts = self.cached_vertices.clone();
            for v in &mut world_verts {
                v.position[0] += self.position.x;
                v.position[1] += self.position.y;
            }
            ctx.add_command(0, world_verts, self.cached_indices.clone());
        }

        // 2. Рендер контента (он уже имеет глобальные координаты)
        if let Some(ref mut content) = self.content {
            content.render(ctx);
        }
    }

    fn children(&self) -> &[Box<dyn RenderBox>] {
        if let Some(ref c) = self.content { std::slice::from_ref(c) } else { &[] }
    }
    fn children_mut(&mut self) -> &mut [Box<dyn RenderBox>] {
        if let Some(ref mut c) = self.content { std::slice::from_mut(c) } else { &mut [] }
    }

    fn hit_test(&self, point: Point) -> bool {
        Rect::new(self.position.x, self.position.y, self.size.width, self.size.height).contains(point)
    }

    fn handle_event(&mut self, event: &Event, _ui: &mut UiManager) -> bool {
        match event {
            Event::PointerMove(p) => {
                let was_hovered = self.is_hovered;
                self.is_hovered = self.hit_test(*p);
                if was_hovered != self.is_hovered { self.dirty = true; }
                self.is_hovered
            }
            Event::Click(p) => {
                if self.hit_test(*p) {
                    if let Some(ref cb) = self.on_click { cb(); }
                    return true;
                }
                false
            }
            _ => false
        }
    }

    fn can_focus(&self) -> bool { true }
    fn set_focused(&mut self, _focused: bool) { self.dirty = true; }
    fn is_focused(&self) -> bool { false }
    
    fn handle_key_down(&mut self, key: Key, _modifiers: KeyboardModifiers) -> bool {
        if matches!(key, Key::Enter | Key::Space) {
            if let Some(ref cb) = self.on_click { cb(); }
            true
        } else { false }
    }
    fn handle_char_input(&mut self, _char: char) -> bool { false }
    fn widget_id(&self) -> Option<WidgetId> { None }
    fn margin(&self) -> EdgeInsets { EdgeInsets::default() }
}
    */