pub mod border;
pub mod horizontal;
pub mod vertical;
pub mod padding;


pub fn border() -> BorderLayout { BorderLayout::new() }
pub fn horizontal() -> HorizontalLayout { HorizontalLayout::new() }
pub fn vertical() -> VerticalLayout { VerticalLayout::new() }
pub fn padding() -> PaddingLayout { PaddingLayout::new() }
//pub fn grid(columns: usize) -> GridLayout { GridLayout::new(columns) }
// и т.д.
