// common/layout_strategy.rs
use crate::common::{render_box::RenderBox, types::{Constraints, LayoutContext, Rect}};

pub trait LayoutStrategy {
    /// Выполнить компоновку детей.
    /// - children: изменяемые ссылки на виджеты (уже реализуют RenderBox).
    /// - constraints: ограничения, переданные родителем.
    /// - ctx: контекст для измерения текста/изображений.
    /// Возвращает список прямоугольников (позиция и размер) для каждого ребёнка.
    /// Порядок прямоугольников соответствует порядку children.
    fn layout(
        &mut self,
        children: &mut [&mut dyn RenderBox],
        constraints: Constraints,
        ctx: &mut dyn LayoutContext,
    ) -> Vec<Rect>;
    
    // (Опционально) Можно вернуть предпочтительный размер контейнера без позиционирования.
    // По умолчанию достаточно метода layout, который сам вычисляет и позиции, и размер.
}