//! Typography system

use alloc::string::{String, ToString};

/// Font families
pub const FONT_TENSION_SANS: &str = "Inter";
pub const FONT_WEAVE_SERIF: &str = "Lora";
pub const FONT_FABRIC_MONO: &str = "JetBrains Mono";
pub const FONT_HAND: &str = "Caveat";

/// Font weights
pub const FONT_WEIGHT_NORMAL: u16 = 400;
pub const FONT_WEIGHT_MEDIUM: u16 = 500;
pub const FONT_WEIGHT_SEMIBOLD: u16 = 600;
pub const FONT_WEIGHT_BOLD: u16 = 700;

/// Font size scale
#[derive(Debug, Clone, Copy)]
pub enum FontSize {
    Xs,    // 12px
    Sm,    // 14px
    Base,  // 16px
    Lg,    // 18px
    Xl,    // 20px
    X2l,   // 24px
    X3l,   // 30px
}

impl FontSize {
    pub fn to_pixels(&self) -> f32 {
        match self {
            FontSize::Xs => 12.0,
            FontSize::Sm => 14.0,
            FontSize::Base => 16.0,
            FontSize::Lg => 18.0,
            FontSize::Xl => 20.0,
            FontSize::X2l => 24.0,
            FontSize::X3l => 30.0,
        }
    }
}

/// Text style definition
#[derive(Debug, Clone)]
pub struct TextStyle {
    pub font_family: String,
    pub font_size: f32,
    pub font_weight: u16,
    pub line_height: f32,
    pub letter_spacing: f32,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_family: FONT_TENSION_SANS.to_string(),
            font_size: 16.0,
            font_weight: FONT_WEIGHT_NORMAL,
            line_height: 1.5,
            letter_spacing: 0.0,
        }
    }
}

impl TextStyle {
    pub fn ui() -> Self {
        Self {
            font_family: FONT_TENSION_SANS.to_string(),
            font_size: 14.0,
            font_weight: FONT_WEIGHT_MEDIUM,
            line_height: 1.5,
            letter_spacing: 0.0,
        }
    }

    pub fn content() -> Self {
        Self {
            font_family: FONT_WEAVE_SERIF.to_string(),
            font_size: 16.0,
            font_weight: FONT_WEIGHT_NORMAL,
            line_height: 1.6,
            letter_spacing: 0.01,
        }
    }

    pub fn mono() -> Self {
        Self {
            font_family: FONT_FABRIC_MONO.to_string(),
            font_size: 14.0,
            font_weight: FONT_WEIGHT_NORMAL,
            line_height: 1.4,
            letter_spacing: 0.0,
        }
    }
}
