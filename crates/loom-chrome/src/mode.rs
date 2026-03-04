//! Mode system - Traditional vs AI-assisted

use loom_core::BrowserMode;

/// Mode manager
#[derive(Debug, Default)]
pub struct ModeManager {
    pub global_mode: BrowserMode,
    pub window_override: Option<BrowserMode>,
}

impl ModeManager {
    pub fn current_mode(&self) -> BrowserMode {
        self.window_override.unwrap_or(self.global_mode)
    }

    pub fn toggle_override(&mut self) {
        let current = self.current_mode();
        self.window_override = Some(match current {
            BrowserMode::Traditional => BrowserMode::AiAssisted,
            BrowserMode::AiAssisted => BrowserMode::Traditional,
        });
    }
}
