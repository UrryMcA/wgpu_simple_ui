use crate::common::render_box::RenderBox;
use crate::common::types::{LayoutContext, Size, EdgeInsets};
use crate::common::render_box::WidgetId;

pub trait Widget {
    fn min_size(&self, ctx: &mut dyn LayoutContext) -> Size;
    fn margin(&self) -> EdgeInsets { EdgeInsets::default() }
    fn padding(&self) -> EdgeInsets { EdgeInsets::default() }
    fn create_render_object(&mut self) -> Box<dyn RenderBox>;

    // ---------- НОВЫЕ МЕТОДЫ (пункт 1 плана) ----------
    /// Устанавливает уникальный идентификатор виджета.
    /// По умолчанию ничего не делает. Виджеты, поддерживающие ID, переопределяют этот метод.
    fn set_id(&mut self, _id: WidgetId) {}

    /// Возвращает идентификатор виджета, если он был установлен.
    /// По умолчанию возвращает `None`.
    fn id(&self) -> Option<WidgetId> {
        None
    }
}

pub trait LeafRenderObjectWidget: Widget {}
pub trait SingleChildRenderObjectWidget: Widget {
    fn child(&self) -> Option<&dyn Widget>;
}
pub trait MultiChildRenderObjectWidget: Widget {
    fn children(&self) -> &[Box<dyn Widget>];
}