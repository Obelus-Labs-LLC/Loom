//! Loom Core - Shared types, constants, and primitives
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod color;
pub mod geometry;
pub mod text;
pub mod tab_manager;
pub mod battery;
pub mod session_restore;
pub mod auto_update;
pub mod onboarding;

pub use color::*;
pub use geometry::*;
pub use text::*;
pub use tab_manager::*;
pub use battery::*;
pub use session_restore::*;
pub use auto_update::*;
pub use onboarding::*;

/// Browser modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BrowserMode {
    #[default]
    Traditional,
    AiAssisted,
}

/// Chromatic temperature for UI theming
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Temperature {
    #[default]
    Auto,
    Warm,
    Cool,
    Neutral,
}

/// Density factor for layout
pub const DENSITY_TRADITIONAL: f32 = 1.0;
pub const DENSITY_AI: f32 = 0.3;

impl Temperature {
    /// Get the current temperature based on auto settings
    /// 
    /// NOTE: On no_std systems without clock access, defaults to Warm
    pub fn resolve(&self) -> Self {
        match self {
            Temperature::Auto => {
                // In no_std environment, we can't access system time
                // Default to warm for morning-like feel
                #[cfg(feature = "std")]
                {
                    use chrono::Timelike;
                    let hour = chrono::Local::now().hour();
                    if hour >= 6 && hour < 12 {
                        Temperature::Warm // Morning
                    } else if hour >= 12 && hour < 18 {
                        Temperature::Neutral // Afternoon
                    } else {
                        Temperature::Cool // Evening/Night
                    }
                }
                #[cfg(not(feature = "std"))]
                {
                    Temperature::Warm // Default for no_std
                }
            }
            other => *other,
        }
    }
}
