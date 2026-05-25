// src/ui/decorated_box.rs
use crate::common::primitives::Primitives;
use crate::common::render_box::{RenderBox, WidgetId};
use crate::common::render_context::RenderContext;
use crate::common::types::*;
use crate::common::vertex::Vertex;
use crate::texture_manager::TextureManager;
use crate::ui_manager::UiManager;
use crate::widgets::canvas::CanvasItem;
use crate::widgets::*;

/// Тип фона для DecoratedBox
pub enum Background {
    /// Заливка сплошным цветом
    Solid(UColor),
    /// Изображение (текстура) с заданным способом вписывания и оттенком
    Image {
        texture_id: u64,
        fit: BackgroundFit,
        tint: UColor,
    },
    /// Набор примитивов Canvas (рисуется как единое целое)
    Canvas(Vec<CanvasItem>),
}

impl Background {
    /// Создать сплошной цвет (белый)
    pub fn white() -> Self {
        Background::Solid(UColor::new(1.0, 1.0, 1.0, 1.0))
    }
}

/// Виджет, который декорирует своего единственного ребёнка фоном, обводкой и скруглениями.
/// Не содержит интерактивной логики (hover, click и т.д.).
pub struct DecoratedBox {
    child: Box<dyn Widget>,
    background: Background,
    corner_radius: f32,
    border: Option<(f32, UColor)>, // толщина, цвет
    margin: EdgeInsets,
    padding: EdgeInsets,
}

impl DecoratedBox {
    pub fn new(child: Box<dyn Widget>) -> Self {
        Self {
            child,
            background: Background::white(),
            corner_radius: 0.0,
            border: None,
            margin: EdgeInsets::default(),
            padding: EdgeInsets::default(),
        }
    }

    pub fn background(mut self, bg: Background) -> Self {
        self.background = bg;
        self
    }

    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }

    pub fn border(mut self, thickness: f32, color: UColor) -> Self {
        self.border = Some((thickness, color));
        self
    }

    pub fn margin(mut self, margin: EdgeInsets) -> Self {
        self.margin = margin;
        self
    }

    pub fn padding(mut self, padding: EdgeInsets) -> Self {
        self.padding = padding;
        self
    }
}

impl Widget for DecoratedBox {
    fn margin(&self) -> EdgeInsets {
        self.margin
    }

    fn padding(&self) -> EdgeInsets {
        self.padding
    }

    fn create_render_object(&mut self) -> Box<dyn RenderBox> {
        let child_ro = self.child.create_render_object();
        Box::new(DecoratedBoxRenderObject {
            child: child_ro,
            background: std::mem::take(&mut self.background),
            corner_radius: self.corner_radius,
            border: self.border,
            margin: self.margin,
            padding: self.padding,
            position: Point::default(),
            size: Size::default(),
            bg_vertices: Vec::new(),
            bg_indices: Vec::new(),
            border_vertices: Vec::new(),
            border_indices: Vec::new(),
            dirty: true,
        })
    }

    fn min_size(&self, ctx: &mut dyn LayoutContext) -> Size {
        let child_min = self.child.min_size(ctx);
        let padded = self.padding.inflate(child_min);
        self.margin.inflate(padded)
    }
}

struct DecoratedBoxRenderObject {
    child: Box<dyn RenderBox>,
    background: Background,
    corner_radius: f32,
    border: Option<(f32, UColor)>,
    margin: EdgeInsets,
    padding: EdgeInsets,
    position: Point,
    size: Size,
    bg_vertices: Vec<Vertex>,
    bg_indices: Vec<u32>,
    border_vertices: Vec<Vertex>,
    border_indices: Vec<u32>,
    dirty: bool,
}

impl DecoratedBoxRenderObject {
    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn rebuild_background_cache(&mut self, primitives: &dyn Primitives, textures: &TextureManager) {
        if self.size.width <= 0.0 || self.size.height <= 0.0 {
            self.bg_vertices.clear();
            self.bg_indices.clear();
            return;
        }

        let rect = Rect::new(0.0, 0.0, self.size.width, self.size.height);

        match &self.background {
            Background::Solid(color) => {
                let (v, i) = primitives.rounded_rect_vertices_indices(rect, self.corner_radius, *color);
                self.bg_vertices = v;
                self.bg_indices = i;
            }
            Background::Image { texture_id, fit, tint } => {
                if let Some(tex_size) = textures.get_size_by_id(*texture_id) {
                    let tex_coords = fit.calculate_tex_coords(&rect, tex_size);
                    let (v, i) = primitives.rounded_textured_rect_vertices_indices(rect, self.corner_radius, tex_coords, *tint);
                    self.bg_vertices = v;
                    self.bg_indices = i;
                } else {
                    // fallback: белый цвет
                    let (v, i) = primitives.rounded_rect_vertices_indices(rect, self.corner_radius, UColor::new(1.0,1.0,1.0,1.0));
                    self.bg_vertices = v;
                    self.bg_indices = i;
                }
            }
            Background::Canvas(items) => {
                let mut tmp_verts = Vec::new();
                let mut tmp_inds = Vec::new();
                for item in items {
                    let (verts, inds) = match item {
                        CanvasItem::Rect { rect, color } => primitives.rect_vertices_indices(rect, color),
                        CanvasItem::RoundedRect { rect, radius, color } => primitives.rounded_rect_vertices_indices(rect, radius, color),
                        CanvasItem::OutlineRect { rect, radius, thickness, color } => primitives.rounded_rect_outline_vertices_indices(rect, radius, thickness, color),
                        CanvasItem::Line { line, color } => primitives.line_vertices_indices(line, color),
                        CanvasItem::Arc { arc, color } => primitives.arc_vertices_indices(arc, color),
                        CanvasItem::FilledSector { sector, color } => primitives.filled_arc_sector_vertices_indices(sector, color),
                        CanvasItem::Custom(f) => f(primitives),
                    };
                    let base = tmp_verts.len() as u32;
                    tmp_verts.extend(verts);
                    tmp_inds.extend(inds.into_iter().map(|i| i + base));
                }
                self.bg_vertices = tmp_verts;
                self.bg_indices = tmp_inds;
            }
        }
    }

    fn rebuild_border_cache(&mut self, primitives: &dyn Primitives) {
        if let Some((thickness, color)) = self.border {
            if thickness > 0.0 && color.0[3] > 0.0 && self.size.width > 0.0 && self.size.height > 0.0 {
                let rect = Rect::new(0.0, 0.0, self.size.width, self.size.height);
                let (v, i) = primitives.rounded_rect_outline_vertices_indices(rect, self.corner_radius, thickness, color);
                self.border_vertices = v;
                self.border_indices = i;
                return;
            }
        }
        self.border_vertices.clear();
        self.border_indices.clear();
    }

    fn rebuild_cache(&mut self, primitives: &dyn Primitives, textures: &TextureManager) {
        self.rebuild_background_cache(primitives, textures);
        self.rebuild_border_cache(primitives);
        self.dirty = false;
    }
}

impl RenderBox for DecoratedBoxRenderObject {
    fn layout(&mut self, constraints: Constraints, ctx: &mut dyn LayoutContext) -> Size {
        // Вычитаем margin и padding для child
        let child_constraints = Constraints {
            min_width: (constraints.min_width - self.margin.left - self.margin.right - self.padding.left - self.padding.right).max(0.0),
            max_width: (constraints.max_width - self.margin.left - self.margin.right - self.padding.left - self.padding.right).max(0.0),
            min_height: (constraints.min_height - self.margin.top - self.margin.bottom - self.padding.top - self.padding.bottom).max(0.0),
            max_height: (constraints.max_height - self.margin.top - self.margin.bottom - self.padding.top - self.padding.bottom).max(0.0),
        };
        let child_size = self.child.layout(child_constraints, ctx);
        let total_size = Size::new(
            child_size.width + self.margin.left + self.margin.right + self.padding.left + self.padding.right,
            child_size.height + self.margin.top + self.margin.bottom + self.padding.top + self.padding.bottom,
        );
        let new_size = constraints.constrain(total_size);
        if new_size != self.size {
            self.size = new_size;
            self.mark_dirty();
        }
        new_size
    }

    fn set_position(&mut self, pos: Point) {
        if self.position != pos {
            self.position = pos;
            // Позиция ребёнка: позиция родителя + margin + padding
            let child_pos = Point::new(
                pos.x + self.margin.left + self.padding.left,
                pos.y + self.margin.top + self.padding.top,
            );
            self.child.set_position(child_pos);
            self.mark_dirty();
        }
    }

    fn position(&self) -> Point {
        self.position
    }

    fn size(&self) -> Size {
        self.size
    }

    fn render(&mut self, ctx: &mut RenderContext) {
        if self.dirty {
            self.rebuild_cache(ctx.primitives, ctx.textures);
        }

        // Рисуем фон
        if !self.bg_vertices.is_empty() {
            let mut world_verts = self.bg_vertices.clone();
            for v in &mut world_verts {
                v.position[0] += self.position.x;
                v.position[1] += self.position.y;
            }
            let texture_id = match &self.background {
                Background::Image { texture_id, .. } => *texture_id,
                _ => 0,
            };
            ctx.add_command(texture_id, world_verts, self.bg_indices.clone());
        }

        // Рисуем обводку
        if !self.border_vertices.is_empty() {
            let mut world_verts = self.border_vertices.clone();
            for v in &mut world_verts {
                v.position[0] += self.position.x;
                v.position[1] += self.position.y;
            }
            ctx.add_command(0, world_verts, self.border_indices.clone());
        }

        // Рисуем ребёнка
        self.child.render(ctx);
    }

    fn children(&self) -> &[Box<dyn RenderBox>] {
        std::slice::from_ref(&self.child)
    }

    fn children_mut(&mut self) -> &mut [Box<dyn RenderBox>] {
        std::slice::from_mut(&mut self.child)
    }

    fn hit_test(&self, point: Point) -> bool {
        let rect = Rect::new(self.position.x, self.position.y, self.size.width, self.size.height);
        rect.contains(point)
    }

    fn handle_event(&mut self, event: &Event, ui: &mut UiManager) -> bool {
        // Просто передаём событие ребёнку, предварительно проверив hit test
        if let Some(point) = event.point() {
            if self.child.hit_test(point) {
                return self.child.handle_event(event, ui);
            }
            false
        } else {
            self.child.handle_event(event, ui)
        }
    }

    fn margin(&self) -> EdgeInsets {
        self.margin
    }

    fn widget_id(&self) -> Option<WidgetId> {
        self.child.widget_id()
    }

    fn can_focus(&self) -> bool {
        self.child.can_focus()
    }

    fn set_focused(&mut self, focused: bool) {
        self.child.set_focused(focused);
    }

    fn is_focused(&self) -> bool {
        self.child.is_focused()
    }

    fn can_drag(&self) -> bool {
        self.child.can_drag()
    }

    fn drag_data(&self) -> Option<DragData> {
        self.child.drag_data()
    }

    fn on_drag_start(&mut self, point: Point) {
        self.child.on_drag_start(point);
    }

    fn on_drag_move(&mut self, point: Point) {
        self.child.on_drag_move(point);
    }

    fn on_drag_end(&mut self, cancelled: bool) {
        self.child.on_drag_end(cancelled);
    }

    fn can_drop(&self, data: &DragData) -> bool {
        self.child.can_drop(data)
    }

    fn on_drag_enter(&mut self, data: &DragData, point: Point) {
        self.child.on_drag_enter(data, point);
    }

    fn on_drag_leave(&mut self) {
        self.child.on_drag_leave();
    }

    fn on_drop(&mut self, data: &DragData, point: Point) {
        self.child.on_drop(data, point);
    }
}
