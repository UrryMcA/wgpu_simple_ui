// src/ui/widgets/button.rs
use crate::common::event::DragData;
use crate::common::types::{BackgroundFit, EdgeInsets, LayoutContext, Size, UColor};
use crate::ui::decorated_box::{Background, DecoratedBox};
use crate::ui::interactive_box::InteractiveBox;
use crate::ui::interactive_state::InteractiveState;
use crate::ui::widgets::{Container, Image, Label};
use crate::common::render_box::RenderBox;
use crate::widgets::Widget;
use crate::widgets::canvas::CanvasItem;

/// Универсальная кнопка, построенная на композиции `InteractiveBox` и `DecoratedBox`.
pub struct Button {
    child: Option<Box<dyn Widget>>,
    background: Option<Background>,
    corner_radius: f32,
    border: Option<(f32, UColor)>,
    margin: EdgeInsets,
    padding: EdgeInsets,
    on_click: Option<Box<dyn FnMut() + Send>>,
    drag_data: Option<DragData>,
    on_state_change: Option<Box<dyn FnMut(&InteractiveState) + Send>>,
}

impl Button {
    /// Создаёт кнопку с произвольным содержимым.
    pub fn new(child: impl Widget + 'static) -> Self {
        Self {
            child: Some(Box::new(child)),
            background: Some(Background::Solid(UColor::new(0.2, 0.3, 0.5, 1.0))),
            corner_radius: 0.0,
            border: None,
            margin: EdgeInsets::default(),
            padding: EdgeInsets::default(),
            on_click: None,
            drag_data: None,
            on_state_change: None,
        }
    }

    /// Создаёт текстовую кнопку.
    pub fn text(text: &str) -> Self {
        Self::new(Label::new(text).font_size(16.0))
    }

    /// Создаёт кнопку-иконку.
    pub fn icon(texture_id: u64, width: f32, height: f32) -> Self {
        Self::new(Image::new(texture_id, width, height))
    }

    /// Создаёт кнопку с иконкой и текстом.
    pub fn icon_text(icon_id: u64, icon_size: Size, text: &str) -> Self {
        let container = Container::horizontal()
            .spacing(8.0)
            .add_child(Box::new(Image::new(icon_id, icon_size.width, icon_size.height)))
            .add_child(Box::new(Label::new(text).font_size(16.0)));
        Self::new(container)
    }

    // ---------- Настройка фона ----------
    pub fn background(mut self, bg: Background) -> Self {
        self.background = Some(bg);
        self
    }

    pub fn solid_color(mut self, color: UColor) -> Self {
        self.background = Some(Background::Solid(color));
        self
    }

    pub fn image(mut self, texture_id: u64, fit: BackgroundFit, tint: UColor) -> Self {
        self.background = Some(Background::Image { texture_id, fit, tint });
        self
    }

    pub fn canvas(mut self, items: Vec<CanvasItem>) -> Self {
        self.background = Some(Background::Canvas(items));
        self
    }

    // ---------- Настройка внешнего вида ----------
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

    // ---------- Интерактивность ----------
    pub fn on_click(mut self, f: impl FnMut() + Send + 'static) -> Self {
        self.on_click = Some(Box::new(f));
        self
    }

    pub fn drag_data(mut self, data: DragData) -> Self {
        self.drag_data = Some(data);
        self
    }

    pub fn on_state_change(mut self, f: impl FnMut(&InteractiveState) + Send + 'static) -> Self {
        self.on_state_change = Some(Box::new(f));
        self
    }
}

impl Widget for Button {
    fn min_size(&self, _ctx: &mut dyn LayoutContext) -> Size {
        Size::zero()
    }

    fn margin(&self) -> EdgeInsets {
        self.margin
    }

    fn padding(&self) -> EdgeInsets {
        self.padding
    }

    fn create_render_object(&mut self) -> Box<dyn RenderBox> {
        let child = self.child.take().expect("Button child already taken");
        let background = self.background.take().expect("Button background already taken");

        let mut decorated = DecoratedBox::new(child)
            .background(background)
            .corner_radius(self.corner_radius)
            .margin(self.margin)
            .padding(self.padding);
        if let Some((thickness, color)) = self.border {
            decorated = decorated.border(thickness, color);
        }

        let mut interactive = InteractiveBox::new(decorated);
        if let Some(on_click) = self.on_click.take() {
            interactive = interactive.on_click(on_click);
        }
        if let Some(drag_data) = self.drag_data.take() {
            interactive = interactive.drag_data(drag_data);
        }
        if let Some(on_state) = self.on_state_change.take() {
            interactive = interactive.on_state_change(on_state);
        }
        interactive.create_render_object()
    }
}