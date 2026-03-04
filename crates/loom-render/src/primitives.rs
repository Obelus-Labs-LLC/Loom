//! Primitive shapes with tension curves

use loom_core::geometry::TensionRect;

/// Renderable primitive
#[derive(Debug)]
pub enum Primitive {
    TensionRect(TensionRect),
}
