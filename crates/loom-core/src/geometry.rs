//! Geometry primitives with tension curve support

use alloc::vec;
use alloc::vec::Vec;
use glam::Vec2;

/// Rectangle with tension curve corners (not simple rounded)
#[derive(Debug, Clone, Copy)]
pub struct TensionRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub tension: f32, // 0 = sharp corners, 1 = maximum organic curve
}

impl TensionRect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            tension: 0.15, // Default organic tension
        }
    }

    pub fn with_tension(mut self, tension: f32) -> Self {
        self.tension = tension;
        self
    }

    /// Generate bezier control points for organic corners
    pub fn generate_path(&self) -> Vec<Vec2> {
        let t = self.tension * self.width.min(self.height) * 0.1;
        let x = self.x;
        let y = self.y;
        let w = self.width;
        let h = self.height;

        // Organic shape with tension curves
        vec![
            Vec2::new(x + t, y),
            Vec2::new(x + w - t, y),
            Vec2::new(x + w, y + t),
            Vec2::new(x + w, y + h - t),
            Vec2::new(x + w - t, y + h),
            Vec2::new(x + t, y + h),
            Vec2::new(x, y + h - t),
            Vec2::new(x, y + t),
        ]
    }

    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.x
            && point.x <= self.x + self.width
            && point.y >= self.y
            && point.y <= self.y + self.height
    }
}

/// Layout box with computed position
#[derive(Debug, Clone, Copy)]
pub struct LayoutBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub margin: EdgeInsets,
    pub padding: EdgeInsets,
    pub border: EdgeInsets,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct EdgeInsets {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl EdgeInsets {
    pub fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    pub fn uniform(value: f32) -> Self {
        Self::new(value, value, value, value)
    }

    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }

    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }
}

/// Size constraints for layout
#[derive(Debug, Clone, Copy)]
pub struct SizeConstraints {
    pub min_width: f32,
    pub max_width: f32,
    pub min_height: f32,
    pub max_height: f32,
}

impl Default for SizeConstraints {
    fn default() -> Self {
        Self {
            min_width: 0.0,
            max_width: f32::INFINITY,
            min_height: 0.0,
            max_height: f32::INFINITY,
        }
    }
}
