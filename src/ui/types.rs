#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}
impl Size {
    pub fn new(w: f32, h: f32) -> Self { Self { width: w, height: h } }
    pub fn inflate(&self, margin: EdgeInsets) -> Self {
        Size {
            width: self.width + margin.left + margin.right,
            height: self.height + margin.top + margin.bottom,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}
impl Point {
    pub fn new(x: f32, y: f32) -> Self { Self { x, y } }
}

#[derive(Debug, Clone, Copy)]
pub struct Constraints {
    pub min_width: f32,
    pub max_width: f32,
    pub min_height: f32,
    pub max_height: f32,
}
impl Constraints {
    pub fn tight(w: f32, h: f32) -> Self {
        Self { min_width: w, max_width: w, min_height: h, max_height: h }
    }
    pub fn loose() -> Self {
        Self { min_width: 0.0, max_width: f32::INFINITY, min_height: 0.0, max_height: f32::INFINITY }
    }
    pub fn constrain(&self, size: Size) -> Size {
        Size::new(
            size.width.clamp(self.min_width, self.max_width),
            size.height.clamp(self.min_height, self.max_height),
        )
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct EdgeInsets {
    pub left: f32, pub right: f32, pub top: f32, pub bottom: f32,
}
impl EdgeInsets {
    pub fn all(v: f32) -> Self { Self { left: v, right: v, top: v, bottom: v } }
    pub fn inflate(&self, size: Size) -> Size {
        Size::new(size.width + self.left + self.right, size.height + self.top + self.bottom)
    }
    pub fn deflate_rect(&self, rect: Rect) -> Rect {
        Rect {
            x: rect.x + self.left,
            y: rect.y + self.top,
            w: rect.w - self.left - self.right,
            h: rect.h - self.top - self.bottom,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: f32, pub y: f32, pub w: f32, pub h: f32,
}
impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self { Self { x, y, w, h } }
}