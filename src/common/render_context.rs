use crate::common::primitives::Primitives;
use crate::common::types::Rect;
use crate::common::vertex::{DrawCommand, Vertex};
use crate::texture_manager::{TextureManager, SamplerKind};
use crate::font_system::FontSystem;
use wgpu::Device;

pub struct RenderContext<'a> {
    pub commands: &'a mut Vec<DrawCommand>,
    pub primitives: &'a dyn Primitives,
    pub textures: &'a mut TextureManager,
    pub font_system: &'a FontSystem,
    pub device: &'a Device,
    scissor_stack: Vec<Rect>,
}

impl<'a> RenderContext<'a> {
    pub fn new(
        commands: &'a mut Vec<DrawCommand>,
        primitives: &'a dyn Primitives,
        textures: &'a mut TextureManager,
        font_system: &'a FontSystem,
        device: &'a Device,
    ) -> Self {
        Self {
            commands,
            primitives,
            textures,
            font_system,
            device,
            scissor_stack: Vec::new(),
        }
    }

    /// Добавляет команду отрисовки с текущим активным scissor-прямоугольником.
    pub fn add_command(
        &mut self,
        texture_id: u64,
        sampler_kind: SamplerKind,
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
    ) {
        let scissor = self.current_scissor();
        self.commands.push(DrawCommand {
            texture_id,
            sampler_kind,
            scissor_rect: scissor,
            vertices,
            indices,
        });
    }

    pub fn push_scissor(&mut self, rect: Rect) {
        self.scissor_stack.push(rect);
    }

    pub fn pop_scissor(&mut self) {
        self.scissor_stack.pop();
    }

    pub fn current_scissor(&self) -> Option<Rect> {
        self.scissor_stack.last().copied()
    }
}