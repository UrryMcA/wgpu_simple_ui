use super::widget::{Widget, LeafRenderObjectWidget};
use crate::common::render_box::RenderBox;
use crate::common::types::*;
use crate::common::{DrawCommand, Primitives};
use crate::texture_manager::TextureManager;
use crate::ui::UiManager;

pub struct Image {
    texture_id: u64,
    size: Size,
    margin: EdgeInsets,
}

impl Image {
    pub fn new(texture_id: u64, width: f32, height: f32) -> Self {
        Self {
            texture_id,
            size: Size::new(width, height),
            margin: EdgeInsets::default(),
        }
    }
    pub fn margin(mut self, m: EdgeInsets) -> Self { self.margin = m; self }
}

impl Widget for Image {
    fn min_size(&self) -> Size { self.size }
    fn margin(&self) -> EdgeInsets { self.margin }
    fn create_render_object(&self) -> Box<dyn RenderBox> {
        Box::new(ImageRenderObject {
            texture_id: self.texture_id,
            size: self.size,
            position: Point::default(),
        })
    }
}

impl LeafRenderObjectWidget for Image {}

struct ImageRenderObject {
    texture_id: u64,
    size: Size,
    position: Point,
}

impl RenderBox for ImageRenderObject {
    fn layout(&mut self, constraints: Constraints) -> Size {
        let s = constraints.constrain(self.size);
        self.size = s;
        s
    }
    fn set_position(&mut self, pos: Point) { self.position = pos; }
    fn position(&self) -> Point { self.position }
    fn size(&self) -> Size { self.size }
    fn render(&self, commands: &mut Vec<DrawCommand>, primitives: &dyn Primitives, _textures: &TextureManager, _ui_manager: &UiManager) {
        let rect = Rect::new(self.position.x, self.position.y, self.size.width, self.size.height);
        let tex_coords = TexCoords::new(0.0, 0.0, 1.0, 1.0);
        let color = UColor::new(1.0, 1.0, 1.0, 1.0); // или UColor([1.0,1.0,1.0,1.0])

        let verts = primitives.textured_rect_vertices(rect, tex_coords, color);

        commands.push(crate::common::DrawCommand { texture_id: self.texture_id, vertices: verts });
    }
}