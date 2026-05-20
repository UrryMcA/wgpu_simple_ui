use super::widget::{Widget, LeafRenderObjectWidget};
use crate::common::render_box::RenderBox;
use crate::common::{self, Primitives};
use crate::common::types::*;
use crate::common::vertex::{DrawCommand};
use crate::generate_text_vertices;
use crate::texture_manager::TextureManager;
use crate::ui::UiManager;

pub struct Button {
    text: String,
    font_name: String,
    padding: EdgeInsets,
    margin: EdgeInsets,
    color: UColor,
    corner_radius: f32,
}

impl Button {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            font_name: "default".into(),
            padding: EdgeInsets::all(8.0),
            margin: EdgeInsets::default(),
            color: common::types::UColor([0.2, 0.3, 0.5, 1.0]),
            corner_radius: 4.0,
        }
    }
    pub fn padding(mut self, p: EdgeInsets) -> Self { self.padding = p; self }
    pub fn margin(mut self, m: EdgeInsets) -> Self { self.margin = m; self }
    pub fn color(mut self, c: UColor) -> Self { self.color = c; self }
    pub fn corner_radius(mut self, r: f32) -> Self { self.corner_radius = r; self }
}

impl Widget for Button {
    fn min_size(&self) -> Size { Size::new(80.0, 30.0) }
    fn margin(&self) -> EdgeInsets { self.margin }
    fn padding(&self) -> EdgeInsets { self.padding }
    fn create_render_object(&self) -> Box<dyn RenderBox> {
        Box::new(ButtonRenderObject {
            text: self.text.clone(),
            font_name: self.font_name.clone(),
            padding: self.padding,
            color: self.color,
            radius: self.corner_radius,
            position: Point::default(),
            size: Size::default(),
        })
    }
}

impl LeafRenderObjectWidget for Button {}

struct ButtonRenderObject {
    text: String,
    font_name: String,
    padding: EdgeInsets,
    color: UColor,
    radius: f32,
    position: Point,
    size: Size,
}

impl RenderBox for ButtonRenderObject {
    fn layout(&mut self, constraints: Constraints) -> Size {
        let inner = Size::new(80.0, 30.0);
        let total = self.padding.inflate(inner);
        let s = constraints.constrain(total);
        self.size = s;
        s
    }
    fn set_position(&mut self, pos: Point) { self.position = pos; }
    fn position(&self) -> Point { self.position }
    fn size(&self) -> Size { self.size }
    fn render(&self, commands: &mut Vec<DrawCommand>, primitives: &dyn Primitives, _textures: &TextureManager, ui_manager: &UiManager) {
        let rect: Rect = Rect::new(self.position.x, self.position.y, self.size.width, self.size.height);
        let bg = primitives.rounded_rect_vertices(rect, self.radius, self.color);
        commands.push(crate::common::DrawCommand { texture_id: 0, vertices: bg });
        if let Some(font) = ui_manager.get_font(&self.font_name) {
            let tv = generate_text_vertices(
                font, &self.text,
                rect.x + self.padding.left,
                rect.y + self.padding.top,
                1.0, 
                [1.0,1.0,1.0,1.0],
                primitives,
            );
            commands.push(crate::common::DrawCommand { texture_id: font.texture_id(), vertices: tv });
        }
    }
}