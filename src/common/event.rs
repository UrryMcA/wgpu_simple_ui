use crate::common::key::Key;
use crate::common::types::Point;
use std::path::PathBuf;

/// Модификаторы клавиатуры
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct KeyboardModifiers(u8);

impl KeyboardModifiers {
    pub const SHIFT: Self = Self(1 << 0);
    pub const CONTROL: Self = Self(1 << 1);
    pub const ALT: Self = Self(1 << 2);
    pub const SUPER: Self = Self(1 << 3);

    pub const fn empty() -> Self {
        Self(0)
    }

    pub fn shift(self) -> bool {
        self.0 & Self::SHIFT.0 != 0
    }

    pub fn control(self) -> bool {
        self.0 & Self::CONTROL.0 != 0
    }

    pub fn alt(self) -> bool {
        self.0 & Self::ALT.0 != 0
    }

    pub fn super_(self) -> bool {
        self.0 & Self::SUPER.0 != 0
    }

    pub fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }

    pub fn remove(&mut self, other: Self) {
        self.0 &= !other.0;
    }
}

/// Кнопка мыши
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u16),
}

/// Данные, перетаскиваемые при Drag & Drop
#[derive(Debug, Clone)]
pub enum DragData {
    Text(String),
    Color([f32; 4]),
    FilePath(PathBuf),
}

impl DragData {
    pub fn as_text(&self) -> Option<&str> {
        match self {
            DragData::Text(s) => Some(s),
            _ => None,
        }
    }
}

/// Все события, которые может обрабатывать UI
#[derive(Debug, Clone)]
pub enum Event {
    PointerDown(Point, MouseButton),
    PointerUp(Point, MouseButton),
    Click(Point, MouseButton),
    PointerMove(Point),

    MouseWheel {
        delta_x: f32,
        delta_y: f32,
        point: Point,
    },

    KeyDown(Key, KeyboardModifiers),
    KeyUp(Key, KeyboardModifiers),
    CharInput(char),

    DragStart {
        point: Point,
        source_id: u64,
        data: DragData,
        button: MouseButton,
    },
    DragMove(Point),
    DragEnd {
        point: Point,
        cancelled: bool,
    },
    DragEnter {
        point: Point,
        data: DragData,
        button: MouseButton,
    },
    DragLeave,
    DragDrop {
        point: Point,
        data: DragData,
        button: MouseButton,
    },
}

impl Event {
    pub fn point(&self) -> Option<Point> {
        match self {
            Event::PointerDown(p, _)
            | Event::PointerUp(p, _)
            | Event::Click(p, _)
            | Event::PointerMove(p)
            | Event::MouseWheel { point: p, .. }
            | Event::DragStart { point: p, .. }
            | Event::DragMove(p)
            | Event::DragEnd { point: p, .. }
            | Event::DragEnter { point: p, .. }
            | Event::DragDrop { point: p, .. } => Some(*p),
            _ => None,
        }
    }

    pub fn mouse_button(&self) -> Option<MouseButton> {
        match self {
            Event::PointerDown(_, b) => Some(*b),
            Event::PointerUp(_, b) => Some(*b),
            Event::Click(_, b) => Some(*b),
            Event::DragStart { button: b, .. } => Some(*b),
            Event::DragEnter { button: b, .. } => Some(*b),
            Event::DragDrop { button: b, .. } => Some(*b),
            _ => None,
        }
    }
}