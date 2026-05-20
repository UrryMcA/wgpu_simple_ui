use crate::common::types::*;
use crate::common::{DrawCommand, Primitives};
use crate::texture_manager::TextureManager;
use crate::ui::UiManager;

pub trait RenderBox {
    fn layout(&mut self, constraints: Constraints) -> Size;
    fn set_position(&mut self, position: Point);
    fn position(&self) -> Point;
    fn size(&self) -> Size;
    fn render(&self, commands: &mut Vec<DrawCommand>, primitives: &dyn Primitives, textures: &TextureManager, ui_manager: &UiManager);
}