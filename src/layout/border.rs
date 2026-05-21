use crate::common::{
    layout_strategy::{LayoutMeasurer, LayoutArranger},
    render_box::RenderBox,
    types::{Constraints, LayoutContext, Rect, Size},
};

#[derive(Clone)]
pub struct BorderLayout {
    pub north_size: Option<f32>,
    pub south_size: Option<f32>,
    pub west_size: Option<f32>,
    pub east_size: Option<f32>,
    pub h_gap: f32,
    pub v_gap: f32,
}

impl BorderLayout {
    pub fn new() -> Self {
        Self {
            north_size: None,
            south_size: None,
            west_size: None,
            east_size: None,
            h_gap: 0.0,
            v_gap: 0.0,
        }
    }
    pub fn with_gaps(mut self, h: f32, v: f32) -> Self {
        self.h_gap = h;
        self.v_gap = v;
        self
    }
    pub fn with_fixed_north(mut self, h: f32) -> Self {
        self.north_size = Some(h);
        self
    }
    pub fn with_fixed_south(mut self, h: f32) -> Self {
        self.south_size = Some(h);
        self
    }
    pub fn with_fixed_west(mut self, w: f32) -> Self {
        self.west_size = Some(w);
        self
    }
    pub fn with_fixed_east(mut self, w: f32) -> Self {
        self.east_size = Some(w);
        self
    }
}

impl LayoutMeasurer for BorderLayout {
    fn measure(
        &mut self,
        children: &mut [&mut dyn RenderBox],
        constraints: Constraints,
        ctx: &mut dyn LayoutContext,
    ) -> Size {
        if children.len() != 5 {
            return Size::zero();
        }
        let loose = Constraints::loose();
        for child in children.iter_mut() {
            child.layout(loose, ctx);
        }
        let north_h = self.north_size.unwrap_or(children[0].size().height);
        let south_h = self.south_size.unwrap_or(children[1].size().height);
        let west_w = self.west_size.unwrap_or(children[2].size().width);
        let east_w = self.east_size.unwrap_or(children[3].size().width);
        let center_w = children[4].size().width;
        let center_h = children[4].size().height;

        let width = west_w + east_w + center_w + self.h_gap * 2.0;
        let height = north_h + south_h + center_h + self.v_gap * 2.0;
        constraints.constrain(Size::new(width, height))
    }
}

impl LayoutArranger for BorderLayout {
    fn arrange(&mut self, children: &mut [&mut dyn RenderBox], inner_rect: Rect) -> Vec<Rect> {
        let mut rects = vec![Rect::new(0.0, 0.0, 0.0, 0.0); children.len()];
        if children.len() != 5 {
            return rects;
        }
        let mut remaining = inner_rect;

        // North
        let north_h = self.north_size.unwrap_or(children[0].size().height);
        let north_rect = Rect::new(
            remaining.x,
            remaining.y,
            remaining.w,
            north_h.min(remaining.h),
        );
        rects[0] = north_rect;
        remaining.y += north_rect.h + self.v_gap;
        remaining.h -= north_rect.h + self.v_gap;

        // South
        let south_h = self.south_size.unwrap_or(children[1].size().height);
        let south_rect = Rect::new(
            remaining.x,
            remaining.y + remaining.h - south_h,
            remaining.w,
            south_h.min(remaining.h),
        );
        rects[1] = south_rect;
        remaining.h -= south_rect.h + self.v_gap;

        // West
        let west_w = self.west_size.unwrap_or(children[2].size().width);
        let west_rect = Rect::new(
            remaining.x,
            remaining.y,
            west_w.min(remaining.w),
            remaining.h,
        );
        rects[2] = west_rect;
        remaining.x += west_rect.w + self.h_gap;
        remaining.w -= west_rect.w + self.h_gap;

        // East
        let east_w = self.east_size.unwrap_or(children[3].size().width);
        let east_rect = Rect::new(
            remaining.x + remaining.w - east_w,
            remaining.y,
            east_w.min(remaining.w),
            remaining.h,
        );
        rects[3] = east_rect;
        remaining.w -= east_rect.w + self.h_gap;

        // Center
        let center_rect = Rect::new(
            remaining.x,
            remaining.y,
            remaining.w.max(0.0),
            remaining.h.max(0.0),
        );
        rects[4] = center_rect;

        rects
    }
}

impl Default for BorderLayout {
    fn default() -> Self {
        Self::new()
    }
}