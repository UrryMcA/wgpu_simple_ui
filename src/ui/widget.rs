// src/widgets/widget.rs
use crate::common::{render_box::RenderBox, types::*};

pub trait Widget {
    fn min_size(&self) -> Size;
    fn margin(&self) -> EdgeInsets { EdgeInsets::default() }
    fn padding(&self) -> EdgeInsets { EdgeInsets::default() }
    fn create_render_object(&mut self) -> Box<dyn RenderBox>;
}

pub trait LeafRenderObjectWidget: Widget {}
pub trait SingleChildRenderObjectWidget: Widget {
    fn child(&self) -> Option<&dyn Widget>;
}
pub trait MultiChildRenderObjectWidget: Widget {
    fn children(&self) -> &[Box<dyn Widget>];
}