// src/widgets/container.rs
use super::widget::{Widget, MultiChildRenderObjectWidget};
use crate::common::render_box::{RenderBox, WidgetId};
use crate::common::{Primitives, types::*};
use crate::common::vertex::DrawCommand;
use crate::common::event::{Event, KeyboardModifiers, DragData};
use crate::common::key::Key;
use crate::texture_manager::TextureManager;
use crate::ui::UiManager;
use crate::ui::layout::{layout_vertical, layout_horizontal, layout_grid};

/// Направление расположения дочерних элементов
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Axis {
    Vertical,
    Horizontal,
    Grid { cols: usize, rows: usize, spacing_x: f32, spacing_y: f32 },
}

/// Контейнер – виджет, который может содержать несколько дочерних виджетов.
pub struct Container {
    children: Vec<Box<dyn Widget>>,
    axis: Axis,
    spacing: f32,
    alignment: Alignment,
    margin: EdgeInsets,
    padding: EdgeInsets,
    color: Option<UColor>,
    corner_radius: f32,
}

impl Container {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            axis: Axis::Vertical,
            spacing: 0.0,
            alignment: Alignment::Start,
            margin: EdgeInsets::default(),
            padding: EdgeInsets::default(),
            color: None,
            corner_radius: 0.0,
        }
    }

    pub fn vertical() -> Self {
        Self::new().axis(Axis::Vertical)
    }

    pub fn horizontal() -> Self {
        Self::new().axis(Axis::Horizontal)
    }

    pub fn grid(cols: usize, rows: usize, spacing_x: f32, spacing_y: f32) -> Self {
        Self::new().axis(Axis::Grid { cols, rows, spacing_x, spacing_y })
    }

    pub fn axis(mut self, axis: Axis) -> Self {
        self.axis = axis;
        self
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
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

    pub fn color(mut self, color: UColor) -> Self {
        self.color = Some(color);
        self
    }

    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }

    pub fn add_child(mut self, child: Box<dyn Widget>) -> Self {
        self.children.push(child);
        self
    }

    pub fn add_children(mut self, children: Vec<Box<dyn Widget>>) -> Self {
        self.children.extend(children);
        self
    }
}

impl Widget for Container {
    fn min_size(&self) -> Size {
        // Минимальный размер контейнера определяется детьми + отступы
        let children_min = self.children.iter().fold(Size::zero(), |acc, child| {
            let s = child.min_size();
            match self.axis {
                Axis::Vertical => Size::new(acc.width.max(s.width), acc.height + s.height),
                Axis::Horizontal => Size::new(acc.width + s.width, acc.height.max(s.height)),
                Axis::Grid { cols, rows, spacing_x, spacing_y } => {
                    // Примитивная оценка: суммарная ширина/высота
                    let total_w = cols as f32 * s.width + (cols - 1) as f32 * spacing_x;
                    let total_h = rows as f32 * s.height + (rows - 1) as f32 * spacing_y;
                    Size::new(total_w, total_h)
                }
            }
        });
        let padded = self.padding.inflate(children_min);
        self.margin.inflate(padded)
    }

    fn margin(&self) -> EdgeInsets {
        self.margin
    }

    fn padding(&self) -> EdgeInsets {
        self.padding
    }

    fn create_render_object(&self) -> Box<dyn RenderBox> {
        let children_render: Vec<Box<dyn RenderBox>> = self.children
            .iter()
            .map(|w| w.create_render_object())
            .collect();
        Box::new(ContainerRenderObject {
            children: children_render,
            axis: self.axis,
            spacing: self.spacing,
            alignment: self.alignment,
            padding: self.padding,
            color: self.color,
            corner_radius: self.corner_radius,
            position: Point::default(),
            size: Size::default(),
            id: None,
        })
    }
}

impl MultiChildRenderObjectWidget for Container {
    fn children(&self) -> &[Box<dyn Widget>] {
        todo!()
    }
}

/// Объект рендеринга контейнера
struct ContainerRenderObject {
    children: Vec<Box<dyn RenderBox>>,
    axis: Axis,
    spacing: f32,
    alignment: Alignment,
    padding: EdgeInsets,
    color: Option<UColor>,
    corner_radius: f32,
    position: Point,
    size: Size,
    id: Option<WidgetId>,
}

impl ContainerRenderObject {
    /// Вычисляет внутреннюю область после применения padding
    fn inner_rect(&self) -> Rect {
        Rect::new(
            self.position.x + self.padding.left,
            self.position.y + self.padding.top,
            self.size.width - self.padding.left - self.padding.right,
            self.size.height - self.padding.top - self.padding.bottom,
        )
    }

    /// Собирает информацию о детях для layout-функций: (ширина, высота, margin)
    fn children_layout_info(&self) -> Vec<(f32, f32, EdgeInsets)> {
        self.children
            .iter()
            .map(|child| {
                let size = child.size();
                // В реальном проекте margin можно хранить в самом виджете, пока заглушка
                let margin = EdgeInsets::default();
                (size.width, size.height, margin)
            })
            .collect()
    }
}

impl RenderBox for ContainerRenderObject {
    // ---------- Обязательные методы ----------
    fn layout(&mut self, constraints: Constraints, ui_manager: &mut UiManager) -> Size {
        // Регистрируем контейнер (если нужно для фокуса)
        if self.id.is_none() {
            self.id = Some(ui_manager.register_widget(self));
        }

        // 1. Сначала вычисляем размеры детей без ограничений (каждый получает loose constraints)
        let child_constraints = Constraints::loose();
        for child in &mut self.children {
            child.layout(child_constraints, ui_manager);
        }

        // 2. Определяем размер контейнера на основе расположения
        let inner_size = match self.axis {
            Axis::Vertical => {
                let total_height: f32 = self.children.iter().map(|c| c.size().height).sum::<f32>()
                    + self.spacing * (self.children.len() - 1) as f32;
                let max_width = self.children.iter().map(|c| c.size().width).fold(0.0, f32::max);
                Size::new(max_width, total_height)
            }
            Axis::Horizontal => {
                let total_width: f32 = self.children.iter().map(|c| c.size().width).sum::<f32>()
                    + self.spacing * (self.children.len() - 1) as f32;
                let max_height = self.children.iter().map(|c| c.size().height).fold(0.0, f32::max);
                Size::new(total_width, max_height)
            }
            Axis::Grid { cols, rows, spacing_x, spacing_y } => {
                let cell_width = self.children.iter().map(|c| c.size().width).fold(0.0, f32::max);
                let cell_height = self.children.iter().map(|c| c.size().height).fold(0.0, f32::max);
                let total_w = cols as f32 * cell_width + (cols - 1) as f32 * spacing_x;
                let total_h = rows as f32 * cell_height + (rows - 1) as f32 * spacing_y;
                Size::new(total_w, total_h)
            }
        };

        let padded_size = self.padding.inflate(inner_size);
        let constrained_size = constraints.constrain(padded_size);
        self.size = constrained_size;

        // 3. Теперь, когда у нас есть финальный размер контейнера, вычисляем позиции детей
        let inner_rect = self.inner_rect();
        let children_info = self.children_layout_info();

        let child_rects = match self.axis {
            Axis::Vertical => layout_vertical(inner_rect, &children_info, self.spacing, self.alignment),
            Axis::Horizontal => layout_horizontal(inner_rect, &children_info, self.spacing, self.alignment),
            Axis::Grid { cols, rows, spacing_x, spacing_y } => {
                layout_grid(inner_rect, &children_info, cols, rows, spacing_x, spacing_y)
            }
        };

        // Устанавливаем позиции детям
        for (child, rect) in self.children.iter_mut().zip(child_rects.iter()) {
            child.set_position(Point::new(rect.x, rect.y));
            // Также можно применить дополнительные ограничения, если нужно
        }

        self.size
    }

    fn set_position(&mut self, pos: Point) {
        self.position = pos;
    }

    fn position(&self) -> Point {
        self.position
    }

    fn size(&self) -> Size {
        self.size
    }

    fn render(
        &self,
        commands: &mut Vec<DrawCommand>,
        primitives: &dyn Primitives,
        textures: &TextureManager,
        ui_manager: &UiManager,
    ) {
        // Рисуем фон, если задан
        if let Some(color) = self.color {
            let rect = Rect::new(self.position.x, self.position.y, self.size.width, self.size.height);
            let bg = primitives.rounded_rect_vertices(rect, self.corner_radius, color);
            commands.push(DrawCommand { texture_id: 0, vertices: bg });
        }

        // Рендерим детей
        for child in &self.children {
            child.render(commands, primitives, textures, ui_manager);
        }
    }

    // ---------- Дерево ----------
    fn children(&self) -> &[Box<dyn RenderBox>] {
        &self.children
    }

    fn children_mut(&mut self) -> &mut [Box<dyn RenderBox>] {
        &mut self.children
    }

    // ---------- Хит-тестирование ----------
    fn hit_test(&self, point: Point) -> bool {
        let rect = Rect::new(self.position.x, self.position.y, self.size.width, self.size.height);
        rect.contains(point)
    }

    // ---------- Обработка событий (передача детям) ----------
    fn handle_event(&mut self, event: &Event, ui_manager: &mut UiManager) -> bool {
        // Сначала проверяем детей в обратном порядке (Z-порядок)
        for child in self.children_mut().iter_mut().rev() {
            if child.hit_test(event.point().unwrap_or(Point::default())) {
                if child.handle_event(event, ui_manager) {
                    return true;
                }
            }
        }
        // Если никто из детей не обработал, контейнер может обработать сам (например, клик на фоне)
        match event {
            Event::Click(_) => {
                // Можно опционально обработать клик на пустом месте контейнера
                false
            }
            _ => false,
        }
    }

    // ---------- Фокус (контейнер делегирует детям) ----------
    fn can_focus(&self) -> bool {
        false // Сам контейнер не получает фокус, но дети могут
    }

    // ---------- Drag & Drop (контейнер как цель) ----------
    fn can_drop(&self, data: &DragData) -> bool {
        // Контейнер может принять перетаскивание, если хотя бы один ребёнок может
        self.children.iter().any(|c| c.can_drop(data))
    }

    fn on_drag_enter(&mut self, data: &DragData, point: Point) {
        // Уведомляем детей, которые под точкой
        if let Some(child) = self.children_mut().iter_mut().find(|c| c.hit_test(point)) {
            child.on_drag_enter(data, point);
        }
    }

    fn on_drag_leave(&mut self) {
        for child in self.children_mut() {
            child.on_drag_leave();
        }
    }

    fn on_drop(&mut self, data: &DragData, point: Point) {
        if let Some(child) = self.children_mut().iter_mut().find(|c| c.hit_test(point) && c.can_drop(data)) {
            child.on_drop(data, point);
        }
    }

    // ---------- Идентификатор ----------
    fn widget_id(&self) -> Option<WidgetId> {
        self.id
    }
}

impl Drop for ContainerRenderObject {
    fn drop(&mut self) {}
}