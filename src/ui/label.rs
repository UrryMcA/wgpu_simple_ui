use super::widget::{Widget, LeafRenderObjectWidget};
use crate::common::render_box::RenderBox;
use crate::common::types::*;
use crate::common::{DrawCommand, Primitives};
use crate::generate_text_vertices;
use crate::texture_manager::TextureManager;
use crate::ui::UiManager;

pub struct Label {
    text: String,
    font_name: String,
    font_size: f32,
    color: [f32;4],
    margin: EdgeInsets,
}

impl Label {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            font_name: "default".into(),
            font_size: 16.0,
            color: [1.0,1.0,1.0,1.0],
            margin: EdgeInsets::default(),
        }
    }
    pub fn font_size(mut self, size: f32) -> Self { self.font_size = size; self }
    pub fn color(mut self, c: [f32;4]) -> Self { self.color = c; self }
    pub fn margin(mut self, m: EdgeInsets) -> Self { self.margin = m; self }
}

impl Widget for Label {
    fn min_size(&self) -> Size { Size::new(self.text.len() as f32 * self.font_size * 0.6, self.font_size) }
    fn margin(&self) -> EdgeInsets { self.margin }
    fn create_render_object(&self) -> Box<dyn RenderBox> {
        Box::new(LabelRenderObject {
            text: self.text.clone(),
            font_name: self.font_name.clone(),
            font_size: self.font_size,
            color: self.color,
            position: Point::default(),
            size: Size::default(),
        })
    }
}

impl LeafRenderObjectWidget for Label {}

struct LabelRenderObject {
    text: String,
    font_name: String,
    font_size: f32,
    color: [f32;4],
    position: Point,
    size: Size,
}

impl RenderBox for LabelRenderObject {
    fn layout(&mut self, constraints: Constraints) -> Size {
        let s = constraints.constrain(Size::new(self.text.len() as f32 * self.font_size * 0.6, self.font_size));
        self.size = s;
        s
    }
    fn set_position(&mut self, pos: Point) { self.position = pos; }
    fn position(&self) -> Point { self.position }
    fn size(&self) -> Size { self.size }
    fn render(&self, commands: &mut Vec<DrawCommand>, primitives: &dyn Primitives, _textures: &TextureManager, ui_manager: &UiManager) {
        if let Some(font) = ui_manager.get_font(&self.font_name) {
            let scale = self.font_size / font.line_height();
            let tv = generate_text_vertices(
                font, &self.text,
                self.position.x, self.position.y,
                scale, self.color,
                primitives,
            );
            commands.push(crate::common::DrawCommand { texture_id: font.texture_id(), vertices: tv });
        }
    }
}