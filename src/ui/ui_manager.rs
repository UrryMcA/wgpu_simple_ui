use std::collections::HashMap;
use super::widget::Widget;
use super::render_box::RenderBox;
use super::types::{Size, Constraints, Point};
use crate::common::{DrawCommand, Primitives};
use crate::texture_manager::TextureManager;
use crate::common::BitmapFont;

pub struct UiManager {
    root_widget: Option<Box<dyn Widget>>,
    root_render: Option<Box<dyn RenderBox>>,
    primitives: Box<dyn Primitives + Send + Sync>,
    fonts: HashMap<String, Box<dyn BitmapFont + Send + Sync>>,
}

impl UiManager {
    pub fn new(primitives: Box<dyn Primitives + Send + Sync>) -> Self {
        Self {
            root_widget: None,
            root_render: None,
            primitives,
            fonts: HashMap::new(),
        }
    }

    pub fn set_root(&mut self, widget: Box<dyn Widget>) {
        self.root_widget = Some(widget);
        self.root_render = self.root_widget.as_ref().map(|w| w.create_render_object());
    }

    pub fn add_font(&mut self, name: String, font: Box<dyn BitmapFont + Send + Sync>) {
        self.fonts.insert(name, font);
    }

    pub fn get_font(&self, name: &str) -> Option<&dyn BitmapFont> {
        self.fonts.get(name).map(|f| f.as_ref())
    }

    pub fn layout(&mut self, screen_size: Size) {
        if let Some(render) = &mut self.root_render {
            let constraints = Constraints::tight(screen_size.width, screen_size.height);
            render.layout(constraints);
            render.set_position(Point::new(0.0, 0.0));
        }
    }

    pub fn render(&self, commands: &mut Vec<DrawCommand>, textures: &TextureManager) {
        if let Some(render) = &self.root_render {
            render.render(commands, self.primitives.as_ref(), textures, self);
        }
    }
}