pub use horizontal::HorizontalLayout;
pub use vertical::VerticalLayout;
pub use border::BorderLayout;
pub use grid::GridLayout;
pub use absolute::AbsoluteLayout;
pub use center::CenterLayout;
pub use padding::PaddingLayout;
pub use stack::StackLayout;

pub fn horizontal() -> HorizontalLayout { HorizontalLayout::new() }
pub fn vertical() -> VerticalLayout { VerticalLayout::new() }
pub fn border() -> BorderLayout { BorderLayout::new() }
pub fn grid(columns: usize) -> GridLayout { GridLayout::new(columns) }
// и т.д.
