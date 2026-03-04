//! Layout engine with density support

use loom_core::{geometry::*, BrowserMode, DENSITY_AI, DENSITY_TRADITIONAL};

/// Layout tree node
#[derive(Debug)]
pub struct LayoutNode {
    pub bounds: LayoutBox,
    pub children: Vec<LayoutNode>,
    pub mode: BrowserMode,
}

impl LayoutNode {
    /// Calculate density factor based on mode
    pub fn density_factor(&self) -> f32 {
        match self.mode {
            BrowserMode::Traditional => DENSITY_TRADITIONAL,
            BrowserMode::AiAssisted => DENSITY_AI,
        }
    }
}

/// Layout engine
pub struct LayoutEngine;

impl LayoutEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn compute_layout(&self, root: &mut LayoutNode, available_width: f32, available_height: f32) {
        root.bounds.width = available_width;
        root.bounds.height = available_height;
    }
}
