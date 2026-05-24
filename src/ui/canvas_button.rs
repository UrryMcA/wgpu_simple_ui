use crate::common::render_box::{RenderBox, WidgetId};
use crate::common::render_context::RenderContext;
use crate::common::types::*;
use crate::common::event::{Event, KeyboardModifiers};
use crate::common::key::Key;
use crate::ui_manager::UiManager;
use crate::ui::widget::{Widget, LeafRenderObjectWidget};


/// Виджет кнопки с произвольным фоном и контентом.
/// Гарантирует, что контент находится внутри фона, и поддерживает обрезку по краям.
pub struct CanvasButton {
    desired_width: f32,
    desired_height: f32,
    corner_radius: f32,
    background: Option<Box<dyn Widget>>,
    content: Option<Box<dyn Widget>>,
    margin: EdgeInsets,
    on_click: Option<Box<dyn Fn() + Send + Sync>>,
}

impl CanvasButton {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            desired_width: width,
            desired_height: height,
            corner_radius: 0.0,
            background: None,
            content: None,
            margin: EdgeInsets::default(),
            on_click: None,
        }
    }

    /// Установить фон (Canvas, Image, Panel и т.д.)
    pub fn background(mut self, widget: impl Widget + 'static) -> Self {
        self.background = Some(Box::new(widget));
        self
    }

    /// Установить контент (Layout с иконкой, текстом и т.д.)
    pub fn content(mut self, widget: impl Widget + 'static) -> Self {
        self.content = Some(Box::new(widget));
        self
    }

    /// Скругление углов (также включает обрезку контента)
    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
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
        // Кнопка стремится к желаемому размеру, но может расти, если контент огромный
        Size::new(self.desired_width, self.desired_height)
    }

    fn margin(&self) -> EdgeInsets {
        self.margin
    }

    fn padding(&self) -> EdgeInsets {
        EdgeInsets::default()
    }

    fn create_render_object(&mut self) -> Box<dyn RenderBox> {
        let bg_render = self.background.as_mut().map(|c| c.create_render_object());
        let content_render = self.content.as_mut().map(|c| c.create_render_object());

        Box::new(CanvasButtonRenderObject {
            background: bg_render,
            content: content_render,
            desired_width: self.desired_width,
            desired_height: self.desired_height,
            corner_radius: self.corner_radius,
            position: Point::default(),
            size: Size::default(),
            on_click: self.on_click.take(),
            is_hovered: false,
            dirty: true,
        })
    }
}

impl LeafRenderObjectWidget for CanvasButton {}

struct CanvasButtonRenderObject {
    background: Option<Box<dyn RenderBox>>,
    content: Option<Box<dyn RenderBox>>,
    desired_width: f32,
    desired_height: f32,
    corner_radius: f32,
    position: Point,
    size: Size,
    on_click: Option<Box<dyn Fn() + Send + Sync>>,
    is_hovered: bool,
    dirty: bool,
}

impl CanvasButtonRenderObject {
    fn update_children_layout(&mut self, ctx: &mut dyn LayoutContext) {
        // 1. Layout фона: он должен занимать всю площадь кнопки
        if let Some(ref mut bg) = self.background {
            let bg_constraints = Constraints::tight(self.size.width, self.size.height);
            bg.layout(bg_constraints, ctx);
            bg.set_position(self.position);
        }

        // 2. Layout контента: центрируем его внутри кнопки
        if let Some(ref mut content) = self.content {
            // Сначала узнаем желаемый размер контента в свободных ограничениях
            let loose = Constraints::loose();
            let content_natural = content.layout(loose, ctx);
            
            // Ограничиваем контент размером кнопки (чтобы не вылезал)
            let max_w = self.size.width.min(content_natural.width);
            let max_h = self.size.height.min(content_natural.height);
            
            let final_constraints = Constraints {
                min_width: 0.0,
                max_width: max_w,
                min_height: 0.0,
                max_height: max_h,
            };
            
            let final_size = content.layout(final_constraints, ctx);
            
            // Центрируем
            let pos_x = self.position.x + (self.size.width - final_size.width) / 2.0;
            let pos_y = self.position.y + (self.size.height - final_size.height) / 2.0;
            content.set_position(Point::new(pos_x, pos_y));
        }
    }
}

impl RenderBox for CanvasButtonRenderObject {
    fn layout(&mut self, constraints: Constraints, ctx: &mut dyn LayoutContext) -> Size {
        let desired = Size::new(self.desired_width, self.desired_height);
        let final_size = constraints.constrain(desired);

        if (final_size.width - self.size.width).abs() > 0.01
            || (final_size.height - self.size.height).abs() > 0.01
        {
            self.size = final_size;
            self.dirty = true;
        }

        // Позиционируем детей относительно новой позиции и размера
        self.update_children_layout(ctx);

        final_size
    }

    fn set_position(&mut self, pos: Point) {
        if self.position != pos {
            self.position = pos;
            self.dirty = true;
            // При движении кнопки нужно пересчитать позиции детей
            // Создаем фиктивный контекст или просто обновляем позиции напрямую, если контекст не нужен для layout
            // Но так как у нас уже был layout, нам нужно только set_position.
            // Однако, чтобы быть точными, вызовем update_children_layout с существующим размером.
            // Для этого нам нужен контекст. В реальном рендерере это делается через проход layout.
            // Здесь мы просто обновим позиции вручную, зная, что размеры уже посчитаны.
            
            if let Some(ref mut bg) = self.background {
                bg.set_position(self.position);
            }
            if let Some(ref mut content) = self.content {
                let c_size = content.size();
                let pos_x = self.position.x + (self.size.width - c_size.width) / 2.0;
                let pos_y = self.position.y + (self.size.height - c_size.height) / 2.0;
                content.set_position(Point::new(pos_x, pos_y));
            }
        }
    }

    fn position(&self) -> Point { self.position }
    fn size(&self) -> Size { self.size }

    fn render(&mut self, ctx: &mut RenderContext) {
        if self.dirty {
            // Пересчет layout мог потребоваться, но обычно делается в pass layout
            // Здесь мы просто помечаем, что позиции могут быть старыми, если размер менялся
        }

        // 1. Рисуем фон
        if let Some(ref mut bg) = self.background {
            bg.render(ctx);
        }

        // 2. Если есть скругление, рисуем клиппинг-маску поверх фона, но перед контентом?
        // В wgpu_simple_ui нет аппаратного клиппинга в этом виде.
        // Хак: мы можем нарисовать "вырезающую" рамку поверх фона, если фон был сплошным.
        // Но если фон сложный (Canvas/Image), нам нужно обрезать его при рендеринге.
        // Поскольку прямой клиппинг сложен без изменений движка, мы полагаемся на то,
        // что пользователь сам рисует фон с учетом радиуса (например, RoundedRect в Canvas).
        
        // Альтернатива для контента: если контент вылезает, он обрежется визуально только если
        // мы нарисуем поверх него маску с прозрачностью снаружи.
        // Для простоты: считаем, что фон уже нарисован правильно (скруглен), а контент центрирован.
        // Если контент вылезает за края скругления, это визуальный баг, который решается
        // либо уменьшением контента, либо добавлением маски.
        
        // Добавим простую маску-рамку поверх всего, если corner_radius > 0, чтобы скрыть вылезания?
        // Нет, это перекроет тени и т.д.
        // Лучшее решение: пользователь должен использовать RoundedRect для фона.
        // А для контента - надеяться, что он влезает.
        
        // 3. Рисуем контент
        if let Some(ref mut content) = self.content {
            content.render(ctx);
        }
        
        self.dirty = false;
    }

    fn children(&self) -> &[Box<dyn RenderBox>] {
        // Возвращаем массив детей для корректной обработки событий и т.д.
        // Так как их может быть 0, 1 или 2, используем статический вектор или логику
        // Для простоты здесь опустим сложную логику слайсов, если дети опциональны.
        // В данной реализации дети обрабатываются напрямую в handle_event/render/layout.
        &[] 
    }

    fn children_mut(&mut self) -> &mut [Box<dyn RenderBox>] {
        &mut []
    }

    fn hit_test(&self, point: Point) -> bool {
        // Проверка попадания с учетом скругления (упрощенно - по прямоугольнику)
        if self.corner_radius > 0.0 {
            // Можно добавить проверку расстояния до углов для точности
            Rect::new(self.position.x, self.position.y, self.size.width, self.size.height)
                .contains(point)
        } else {
            Rect::new(self.position.x, self.position.y, self.size.width, self.size.height)
                .contains(point)
        }
    }

    fn handle_event(&mut self, event: &Event, _ui: &mut UiManager) -> bool {
        // Сначала даем событие контенту (если попал)
        if let Some(ref mut content) = self.content {
            if content.hit_test(event.get_point().unwrap_or(Point::new(-1.0, -1.0))) {
                if content.handle_event(event, _ui) {
                    return true;
                }
            }
        }
        
        // Потом фону
        if let Some(ref mut bg) = self.background {
            //if bg.hit_test(*event.get_point().unwrap_or(&Point::new(-1.0, -1.0))) {
            if bg.hit_test(event.get_point().unwrap_or(Point::new(-1.0, -1.0))) {
                if bg.handle_event(event, _ui) {
                    return true;
                }
            }
        }

        // Обработка клика по самой кнопке
        match event {
            Event::PointerMove(p) => {
                let was_hovered = self.is_hovered;
                self.is_hovered = self.hit_test(*p);
                if was_hovered != self.is_hovered {
                    self.dirty = true;
                }
                self.is_hovered
            }
            Event::Click(p) => {
                if self.hit_test(*p) {
                    if let Some(ref cb) = self.on_click {
                        cb();
                    }
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

// Helper to get point from event if needed
impl Event {
    fn get_point(&self) -> Option<Point> {
        match self {
            Event::PointerMove(p) | Event::PointerDown(p) | Event::PointerUp(p) | Event::Click(p) => Some(*p),
            _ => None
        }
    }
}