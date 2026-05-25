// src/widgets/canvas.rs
use crate::common::render_box::{RenderBox, WidgetId};
use crate::common::render_context::RenderContext;
use crate::common::types::*;
use crate::common::vertex::Vertex;
use crate::common::primitives::Primitives;
use crate::common::event::{Event, KeyboardModifiers};
use crate::common::key::Key;
use crate::ui_manager::UiManager;
use crate::texture_manager::SamplerKind;
use crate::widgets::{LeafRenderObjectWidget, Widget};

/// Примитив для отрисовки на Canvas
pub enum CanvasItem {
    Rect { rect: Rect, color: UColor },
    RoundedRect { rect: Rect, radius: f32, color: UColor },
    OutlineRect { rect: Rect, radius: f32, thickness: f32, color: UColor },
    Line { line: Line, color: UColor },
    Arc { arc: Arc, color: UColor },
    FilledSector { sector: FilledArcSector, color: UColor },
    /// Пользовательский callback для экзотики
    Custom(Box<dyn Fn(&dyn Primitives) -> (Vec<Vertex>, Vec<u32>) + Send + Sync>),
}

pub struct Canvas {
    pub width: f32,
    pub height: f32,
    pub items: Vec<CanvasItem>,
    margin: EdgeInsets,
    /// Опциональный callback для hit-testing по координатам
    on_click: Option<Box<dyn FnMut(Point) + Send>>,
}

impl Canvas {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            items: Vec::new(),
            margin: EdgeInsets::default(),
            on_click: None,
        }
    }

    pub fn add_item(mut self, item: CanvasItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn push_item(&mut self, item: CanvasItem) {
        self.items.push(item);
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    pub fn margin(mut self, m: EdgeInsets) -> Self {
        self.margin = m;
        self
    }

    pub fn on_click(mut self, f: impl FnMut(Point) + Send + 'static) -> Self {
        self.on_click = Some(Box::new(f));
        self
    }
}

impl Widget for Canvas {
    fn min_size(&self, _ctx: &mut dyn LayoutContext) -> Size {
        Size::new(self.width, self.height)
    }
    fn margin(&self) -> EdgeInsets { self.margin }
    fn padding(&self) -> EdgeInsets { EdgeInsets::default() }
    fn create_render_object(&mut self) -> Box<dyn RenderBox> {
        Box::new(CanvasRenderObject {
            items: std::mem::take(&mut self.items),
            width: self.width,
            height: self.height,
            position: Point::default(),
            size: Size::default(),
            on_click: self.on_click.take(),
            cached_vertices: Vec::new(),
            cached_indices: Vec::new(),
            dirty: true,
        })
    }
}

impl  LeafRenderObjectWidget for Canvas {}

struct CanvasRenderObject {
    items: Vec<CanvasItem>,
    width: f32,
    height: f32,
    position: Point,
    size: Size,
    on_click: Option<Box<dyn FnMut(Point) + Send>>,
    cached_vertices: Vec<Vertex>,
    cached_indices: Vec<u32>,
    dirty: bool,
}

impl CanvasRenderObject {
    fn rebuild_cache(&mut self, primitives: &dyn Primitives) {
        if self.size.width < 1.0 || self.size.height < 1.0 {
            return;
        }

        self.cached_vertices.clear();
        self.cached_indices.clear();

        for item in &self.items {
            let (verts, inds) = match item {
                CanvasItem::Rect { rect, color } =>
                    primitives.rect_vertices_indices(*rect, *color),
                CanvasItem::RoundedRect { rect, radius, color } =>
                    primitives.rounded_rect_vertices_indices(*rect, *radius, *color),
                CanvasItem::OutlineRect { rect, radius, thickness, color } =>
                    primitives.rounded_rect_outline_vertices_indices(*rect, *radius, *thickness, *color),
                CanvasItem::Line { line, color } =>
                    primitives.line_vertices_indices(*line, *color),
                CanvasItem::Arc { arc, color } =>
                    primitives.arc_vertices_indices(*arc, *color),
                CanvasItem::FilledSector { sector, color } =>
                    primitives.filled_arc_sector_vertices_indices(*sector, *color),
                CanvasItem::Custom(f) => f(primitives),
            };

            let base = self.cached_vertices.len() as u32;
            self.cached_vertices.extend(verts);
            self.cached_indices.extend(inds.into_iter().map(|i| i + base));
        }

        self.dirty = false;
    }
}

impl RenderBox for CanvasRenderObject {
    fn layout(&mut self, constraints: Constraints, _ctx: &mut dyn LayoutContext) -> Size {
        let desired = Size::new(self.width, self.height);
        let new_size = constraints.constrain(desired);
        if (new_size.width - self.size.width).abs() > 0.01
            || (new_size.height - self.size.height).abs() > 0.01
        {
            self.size = new_size;
            self.dirty = true;
        }
        new_size
    }

    fn set_position(&mut self, pos: Point) {
        if self.position != pos {
            self.position = pos;
            self.dirty = true;
        }
    }

    fn position(&self) -> Point { self.position }
    fn size(&self) -> Size { self.size }

    fn render(&mut self, ctx: &mut RenderContext) {
        if self.dirty {
            self.rebuild_cache(ctx.primitives);
        }

        if self.cached_vertices.is_empty() {
            return;
        }

        // Сдвигаем в мировые координаты
        let mut world_verts = self.cached_vertices.clone();
        for v in &mut world_verts {
            v.position[0] += self.position.x;
            v.position[1] += self.position.y;
        }

        // Canvas использует текстуру 0 (сплошные цвета) и сэмплер Clamp
        ctx.add_command(0, SamplerKind::Clamp, world_verts, self.cached_indices.clone());
    }

    fn children(&self) -> &[Box<dyn RenderBox>] { &[] }
    fn children_mut(&mut self) -> &mut [Box<dyn RenderBox>] { &mut [] }

    fn hit_test(&self, point: Point) -> bool {
        Rect::new(self.position.x, self.position.y, self.size.width, self.size.height)
            .contains(point)
    }

    fn handle_event(&mut self, event: &Event, _ui: &mut UiManager) -> bool {
        if let Event::Click(p) = event {
            if let Some(cb) = &mut self.on_click {
                let local = Point::new(p.x - self.position.x, p.y - self.position.y);
                cb(local);
            }
            true
        } else {
            false
        }
    }

    fn can_focus(&self) -> bool { false }
    fn set_focused(&mut self, _: bool) {}
    fn is_focused(&self) -> bool { false }
    fn handle_key_down(&mut self, _: Key, _: KeyboardModifiers) -> bool { false }
    fn handle_char_input(&mut self, _: char) -> bool { false }
    fn widget_id(&self) -> Option<WidgetId> { None }
    fn margin(&self) -> EdgeInsets { EdgeInsets::default() }
}