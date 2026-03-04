//! Text rendering with cosmic-text

use cosmic_text::{Buffer, FontSystem, SwashCache};

/// Text renderer
pub struct TextRenderer {
    font_system: FontSystem,
    swash_cache: SwashCache,
}

impl TextRenderer {
    pub fn new() -> Self {
        Self {
            font_system: FontSystem::new(),
            swash_cache: SwashCache::new(),
        }
    }
}
