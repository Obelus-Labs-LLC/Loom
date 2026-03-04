//! Color system with chromatic temperature support

use glam::Vec4;

/// OKLCH color representation
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Oklch {
    pub l: f32, // Lightness (0-1)
    pub c: f32, // Chroma
    pub h: f32, // Hue (degrees)
}

impl Oklch {
    pub const fn new(l: f32, c: f32, h: f32) -> Self {
        Self { l, c, h }
    }

    /// Convert to RGB (simplified, for exact conversion use color science crate)
    pub fn to_rgb(&self) -> [u8; 4] {
        // Simplified conversion for now
        let l = self.l.clamp(0.0, 1.0);
        let gray = (l * 255.0) as u8;
        [gray, gray, gray, 255]
    }

    pub fn to_vec4(&self) -> Vec4 {
        let [r, g, b, a] = self.to_rgb();
        Vec4::new(
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        )
    }
}

/// Temperature scales
pub mod temperature {
    use super::Oklch;

    // Warm (amber tones)
    pub const WARM_50: Oklch = Oklch::new(0.98, 0.01, 60.0);
    pub const WARM_100: Oklch = Oklch::new(0.95, 0.03, 60.0);
    pub const WARM_200: Oklch = Oklch::new(0.90, 0.05, 55.0);
    pub const WARM_300: Oklch = Oklch::new(0.82, 0.08, 50.0);
    pub const WARM_400: Oklch = Oklch::new(0.75, 0.12, 45.0);
    pub const WARM_500: Oklch = Oklch::new(0.68, 0.15, 40.0);
    pub const WARM_600: Oklch = Oklch::new(0.60, 0.14, 35.0);

    // Cool (indigo tones)
    pub const COOL_50: Oklch = Oklch::new(0.98, 0.01, 240.0);
    pub const COOL_100: Oklch = Oklch::new(0.95, 0.02, 240.0);
    pub const COOL_200: Oklch = Oklch::new(0.88, 0.04, 240.0);
    pub const COOL_300: Oklch = Oklch::new(0.78, 0.06, 240.0);
    pub const COOL_400: Oklch = Oklch::new(0.68, 0.09, 240.0);
    pub const COOL_500: Oklch = Oklch::new(0.58, 0.11, 240.0);
    pub const COOL_600: Oklch = Oklch::new(0.48, 0.10, 240.0);

    // Neutral (silver tones)
    pub const NEUTRAL_50: Oklch = Oklch::new(0.98, 0.005, 0.0);
    pub const NEUTRAL_100: Oklch = Oklch::new(0.95, 0.005, 0.0);
    pub const NEUTRAL_200: Oklch = Oklch::new(0.88, 0.005, 0.0);
    pub const NEUTRAL_300: Oklch = Oklch::new(0.78, 0.005, 0.0);
    pub const NEUTRAL_400: Oklch = Oklch::new(0.68, 0.005, 0.0);
    pub const NEUTRAL_500: Oklch = Oklch::new(0.58, 0.005, 0.0);
    pub const NEUTRAL_600: Oklch = Oklch::new(0.48, 0.005, 0.0);
    pub const NEUTRAL_700: Oklch = Oklch::new(0.38, 0.005, 0.0);
    pub const NEUTRAL_800: Oklch = Oklch::new(0.28, 0.005, 0.0);
    pub const NEUTRAL_900: Oklch = Oklch::new(0.18, 0.005, 0.0);

    // Alert (crimson pulse)
    pub const ALERT: Oklch = Oklch::new(0.55, 0.22, 15.0);
}

/// Theme colors based on temperature
#[derive(Debug, Clone)]
pub struct ThemeColors {
    pub bg_primary: Oklch,
    pub bg_secondary: Oklch,
    pub surface: Oklch,
    pub border: Oklch,
    pub text_primary: Oklch,
    pub text_secondary: Oklch,
    pub text_muted: Oklch,
    pub alert: Oklch,
}

impl Default for ThemeColors {
    fn default() -> Self {
        use temperature::*;
        Self {
            bg_primary: WARM_50,
            bg_secondary: WARM_100,
            surface: WARM_200,
            border: WARM_300,
            text_primary: NEUTRAL_900,
            text_secondary: NEUTRAL_600,
            text_muted: NEUTRAL_400,
            alert: ALERT,
        }
    }
}
