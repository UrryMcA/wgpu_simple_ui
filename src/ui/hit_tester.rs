use crate::common::types::Rect;

// ------------------------------------------------------------
// HitTester для мыши
// ------------------------------------------------------------
pub struct HitTester  {
    areas: Vec<(u32, Rect, Box<dyn FnMut() + Send + Sync>)>,
    hovered_id: Option<u32>,
}

impl HitTester {
    pub fn new() -> Self {
        Self { areas: Vec::new(), hovered_id: None }
    }

    pub fn register(&mut self, id: u32, rect: Rect, callback: Box<dyn FnMut() + Send + Sync>) {
        self.areas.push((id, rect, callback));
    }

    pub fn on_mouse_move(&mut self, x: f32, y: f32) {
        let new_hover = self.areas.iter().find(|(_, rect, _)| rect.contains(x, y)).map(|(id, ..)| *id);
        if self.hovered_id != new_hover {
            self.hovered_id = new_hover;
        }
    }

    pub fn on_mouse_click(&mut self, x: f32, y: f32) {
        if let Some((_, _, callback)) = self.areas.iter_mut().find(|(_, rect, _)| rect.contains(x, y)) {
            callback();
        }
    }

    pub fn hovered_id(&self) -> Option<u32> {
        self.hovered_id
    }
}
