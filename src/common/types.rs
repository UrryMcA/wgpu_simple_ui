use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: f32, pub y: f32, pub w: f32, pub h: f32,
}
impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self { Self { x, y, w, h } }
    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.x && point.x <= self.x + self.w && point.y >= self.y && point.y <= self.y + self.h
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GlyphInfo {
    pub width: f32,
    pub height: f32,
    pub u0: f32, pub v0: f32,
    pub u1: f32, pub v1: f32,
    pub xoffset: f32,
    pub yoffset: f32,
    pub xadvance: f32,
}

pub trait BitmapFont {
    fn texture_id(&self) -> u64;
    fn line_height(&self) -> f32;
    fn get_glyph(&self, ch: char) -> Option<GlyphInfo>;
}

/// Параметры текстурных координат.
#[derive(Debug, Clone, Copy)]
pub struct TexCoords {
    pub u0: f32,
    pub v0: f32,
    pub u1: f32,
    pub v1: f32,
}
impl TexCoords {
    pub fn new(u0: f32, v0: f32, u1: f32, v1: f32) -> Self {
        Self { u0, v0, u1, v1 }
    }
}

/// Параметры цвета.
#[derive(Debug, Clone, Copy)]
pub struct UColor(pub [f32; 4]);
impl UColor {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self([r, g, b, a])
    }
}

/// Линия, заданная двумя точками и толщиной
#[derive(Debug, Clone, Copy)]
pub struct Line {
    pub x1: f32, pub y1: f32,
    pub x2: f32, pub y2: f32,
    pub thickness: f32,
}
impl Line {
    pub fn new(x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32) -> Self {
        Self { x1, y1, x2, y2, thickness }
    }
}

/// Дуга (сектор окружности) – незамкнутая линия
#[derive(Debug, Clone, Copy)]
pub struct Arc {
    pub cx: f32, pub cy: f32,   // центр
    pub r: f32,                 // радиус
    pub start_angle: f32,       // радианы
    pub end_angle: f32,
    pub thickness: f32,
}
impl Arc {
    pub fn new(cx: f32, cy: f32, r: f32, start_angle: f32, end_angle: f32, thickness: f32) -> Self {
        Self { cx, cy, r, start_angle, end_angle, thickness }
    }
}

/// Залитый сектор (как кусок пиццы)
#[derive(Debug, Clone, Copy)]
pub struct FilledArcSector {
    pub cx: f32, pub cy: f32,
    pub r: f32,
    pub start_angle: f32,
    pub end_angle: f32,
}
impl FilledArcSector {
    pub fn new(cx: f32, cy: f32, r: f32, start_angle: f32, end_angle: f32) -> Self {
        Self { cx, cy, r, start_angle, end_angle }
    }
}

/// Скруглённый прямоугольник
#[derive(Debug, Clone, Copy)]
pub struct RoundedRect {
    pub rect: Rect,
    pub radius: f32,
}

/// Контур скруглённого прямоугольника (с дополнительной толщиной)
#[derive(Debug, Clone, Copy)]
pub struct RoundedRectOutline {
    pub rect: Rect,
    pub radius: f32,
    pub thickness: f32,
}

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
    pub fn zero() -> Self { Self { width: 0.0, height: 0.0 } }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}
impl Point {
    pub fn new(x: f32, y: f32) -> Self { Self { x, y } }
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Alignment {
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Axis {
    Vertical,
    Horizontal,
    Grid { cols: usize, rows: usize, spacing_x: f32, spacing_y: f32 },
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

/// Сырой глиф (из .fnt файла). Не зависит от serde.
#[derive(Debug, Clone)]
pub struct RawGlyph {
    pub id: u32,
    pub width: u16,
    pub height: u16,
    pub xoffset: i16,
    pub yoffset: i16,
    pub xadvance: i16,
    pub x: u16,
    pub y: u16,
}

pub struct GpuBitmapFont {
    texture_id: u64,
    line_height: f32,
    chars: HashMap<u32, GlyphInfo>,
}

impl GpuBitmapFont {
    pub fn new(texture_id: u64, line_height: f32, chars: HashMap<u32, GlyphInfo>) -> Self {
        Self { texture_id, line_height, chars }
    }
}

impl BitmapFont for GpuBitmapFont {
    fn texture_id(&self) -> u64 { self.texture_id }
    fn line_height(&self) -> f32 { self.line_height }
    fn get_glyph(&self, ch: char) -> Option<GlyphInfo> {
        self.chars.get(&(ch as u32)).copied()
    }
}


/// Загрузчик текстур: возвращает RGBA данные и размеры.
pub trait TextureLoader {
    fn load_texture_rgba(&self, name: &str) -> Option<(Vec<u8>, u32, u32)>;
}

/// Загрузчик шрифтов: возвращает RGBA атласа, размеры атласа и список сырых глифов.
pub trait FontLoader {
    fn load_font_data(&self, name: &str) -> Option<(Vec<u8>, u32, u32, Vec<RawGlyph>)>;
}

pub trait LayoutContext {
    /// Измерить текст с заданным шрифтом и ограничением по ширине.
    fn measure_text(&mut self, text: &str, font_size: f32, max_width: f32) -> Size;
    
    /// Получить размер изображения по пути (если загружено).
    fn get_image_size(&mut self, path: &str) -> Option<Size>;
    
    /// (Опционально) доступ к глобальным стилям, масштабу и т.д.
    fn scale_factor(&self) -> f32;
}
