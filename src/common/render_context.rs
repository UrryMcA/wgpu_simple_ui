// src/common/render_context.rs
use crate::common::primitives::Primitives;
use crate::common::types::Rect;
use crate::common::vertex::{DrawCommand, Vertex};
use crate::texture_manager::TextureManager;
use crate::font_system::FontSystem;

/// Контекст рендеринга, передаваемый в `RenderBox::render`.
/// Содержит все необходимые данные для добавления команд отрисовки
/// и управления стеком scissor-областей.
pub struct RenderContext<'a> {
    pub commands: &'a mut Vec<DrawCommand>,
    pub primitives: &'a dyn Primitives,
    pub textures: &'a TextureManager,
    pub font_system: &'a FontSystem,
    scissor_stack: Vec<Rect>,
}

impl<'a> RenderContext<'a> {
    pub fn new(
        commands: &'a mut Vec<DrawCommand>,
        primitives: &'a dyn Primitives,
        textures: &'a TextureManager,
        font_system: &'a FontSystem,
    ) -> Self {
        Self {
            commands,
            primitives,
            textures,
            font_system,
            scissor_stack: Vec::new(),
        }
    }

    /// Добавляет команду отрисовки с текущим активным scissor-прямоугольником.
    pub fn add_command(&mut self, texture_id: u64, vertices: Vec<Vertex>) {
        let scissor = self.current_scissor();
        self.commands.push(DrawCommand {
            texture_id,
            vertices,
            scissor_rect: scissor,
        });
    }

    // ---------- Scissor стек ----------
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