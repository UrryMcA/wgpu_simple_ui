// src/layout/border.rs
use crate::common::{
    layout_strategy::{LayoutMeasurer, LayoutArranger},
    render_box::RenderBox,
    types::{Constraints, LayoutContext, Rect, Size, EdgeInsets},
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
    pub fn with_gaps(mut self, h: f32, v: f32) -> Self { self.h_gap = h; self.v_gap = v; self }
    pub fn with_fixed_north(mut self, h: f32) -> Self { self.north_size = Some(h); self }
    pub fn with_fixed_south(mut self, h: f32) -> Self { self.south_size = Some(h); self }
    pub fn with_fixed_west(mut self, w: f32) -> Self { self.west_size = Some(w); self }
    pub fn with_fixed_east(mut self, w: f32) -> Self { self.east_size = Some(w); self }
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

        // Учитываем margin при вычислении размеров регионов
        let north_margin = children[0].margin();
        let south_margin = children[1].margin();
        let west_margin = children[2].margin();
        let east_margin = children[3].margin();
        let center_margin = children[4].margin();

        let north_h = self.north_size.unwrap_or(children[0].size().height + north_margin.top + north_margin.bottom);
        let south_h = self.south_size.unwrap_or(children[1].size().height + south_margin.top + south_margin.bottom);
        let west_w = self.west_size.unwrap_or(children[2].size().width + west_margin.left + west_margin.right);
        let east_w = self.east_size.unwrap_or(children[3].size().width + east_margin.left + east_margin.right);
        let center_w = children[4].size().width + center_margin.left + center_margin.right;
        let center_h = children[4].size().height + center_margin.top + center_margin.bottom;

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
        let north_margin = children[0].margin();
        let north_h = self.north_size.unwrap_or(children[0].size().height + north_margin.top + north_margin.bottom);
        let north_rect = Rect::new(
            remaining.x + north_margin.left,
            remaining.y + north_margin.top,
            (remaining.w - north_margin.left - north_margin.right).max(0.0),
            (north_h - north_margin.top - north_margin.bottom).max(0.0).min(remaining.h),
        );
        rects[0] = north_rect;
        remaining.y += north_rect.h + self.v_gap + north_margin.top + north_margin.bottom;
        remaining.h -= north_rect.h + self.v_gap + north_margin.top + north_margin.bottom;

        // South
        let south_margin = children[1].margin();
        let south_h = self.south_size.unwrap_or(children[1].size().height + south_margin.top + south_margin.bottom);
        let south_rect = Rect::new(
            remaining.x + south_margin.left,
            remaining.y + remaining.h - south_h + south_margin.top,
            (remaining.w - south_margin.left - south_margin.right).max(0.0),
            (south_h - south_margin.top - south_margin.bottom).max(0.0).min(remaining.h),
        );
        rects[1] = south_rect;
        remaining.h -= south_rect.h + self.v_gap + south_margin.top + south_margin.bottom;

        // West
        let west_margin = children[2].margin();
        let west_w = self.west_size.unwrap_or(children[2].size().width + west_margin.left + west_margin.right);
        let west_rect = Rect::new(
            remaining.x + west_margin.left,
            remaining.y + west_margin.top,
            (west_w - west_margin.left - west_margin.right).max(0.0).min(remaining.w),
            (remaining.h - west_margin.top - west_margin.bottom).max(0.0),
        );
        rects[2] = west_rect;
        remaining.x += west_rect.w + self.h_gap + west_margin.left + west_margin.right;
        remaining.w -= west_rect.w + self.h_gap + west_margin.left + west_margin.right;

        // East
        let east_margin = children[3].margin();
        let east_w = self.east_size.unwrap_or(children[3].size().width + east_margin.left + east_margin.right);
        let east_rect = Rect::new(
            remaining.x + remaining.w - east_w + east_margin.left,
            remaining.y + east_margin.top,
            (east_w - east_margin.left - east_margin.right).max(0.0).min(remaining.w),
            (remaining.h - east_margin.top - east_margin.bottom).max(0.0),
        );
        rects[3] = east_rect;
        remaining.w -= east_rect.w + self.h_gap + east_margin.left + east_margin.right;

        // Center
        let center_margin = children[4].margin();
        let center_rect = Rect::new(
            remaining.x + center_margin.left,
            remaining.y + center_margin.top,
            (remaining.w - center_margin.left - center_margin.right).max(0.0),
            (remaining.h - center_margin.top - center_margin.bottom).max(0.0),
        );
        rects[4] = center_rect;

        rects
    }
}

impl Default for BorderLayout {
    fn default() -> Self { Self::new() }
}