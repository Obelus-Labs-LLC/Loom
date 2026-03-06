//! Loom Render - wgpu rendering and text
//!
//! L29: Performance Optimization - React Fiber-style work loop for 60fps

pub mod renderer;
pub mod text;
pub mod primitives;

// L29 Performance modules
pub mod profiler;
pub mod fiber;
pub mod work_loop;
pub mod benchmark;
pub mod ai_mode_transition;

pub use renderer::*;
pub use profiler::*;
pub use fiber::*;
pub use work_loop::*;
pub use benchmark::*;
pub use ai_mode_transition::*;
