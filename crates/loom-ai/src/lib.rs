//! Loom AI - AI-Native Mode
//!
//! Phase L18: AI-Native Mode
//! - Intent parser (natural language to browser actions)
//! - Minimal chrome UI (address bar only, no toolbar)
//! - Agent integration hooks (Concierge connection)
//! - Mode switch: Traditional ↔ AI-Native persists per session
//! - Voice input stub (prepare for L12.5 integration)

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;

pub mod agent;
pub mod chrome;
pub mod intent;
pub mod voice;

pub use agent::*;
pub use chrome::*;
pub use intent::*;
pub use voice::*;

/// Version of the AI crate
pub const VERSION: &str = "0.1.0-L18";

/// Browser mode (Traditional vs AI-Native)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum BrowserMode {
    /// Traditional mode - full chrome, all controls
    #[default]
    Traditional,
    /// AI-Native mode - minimal chrome, intent-driven
    AiNative,
}

impl BrowserMode {
    /// Get mode name
    pub fn name(&self) -> &'static str {
        match self {
            BrowserMode::Traditional => "Traditional",
            BrowserMode::AiNative => "AI-Native",
        }
    }

    /// Check if AI-Native mode
    pub fn is_ai_native(&self) -> bool {
        matches!(self, BrowserMode::AiNative)
    }

    /// Toggle mode
    pub fn toggle(&mut self) {
        *self = match self {
            BrowserMode::Traditional => BrowserMode::AiNative,
            BrowserMode::AiNative => BrowserMode::Traditional,
        };
    }
}

impl fmt::Display for BrowserMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Session configuration that persists mode preference
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionConfig {
    /// Current browser mode
    pub mode: BrowserMode,
    /// AI assistant enabled
    pub ai_enabled: bool,
    /// Voice input enabled
    pub voice_enabled: bool,
    /// Session ID
    pub session_id: String,
}

impl SessionConfig {
    /// Create new session with default mode
    pub fn new(session_id: impl Into<String>) -> Self {
        Self {
            mode: BrowserMode::default(),
            ai_enabled: true,
            voice_enabled: false,
            session_id: session_id.into(),
        }
    }

    /// Create new session with AI-Native mode
    pub fn ai_native(session_id: impl Into<String>) -> Self {
        Self {
            mode: BrowserMode::AiNative,
            ai_enabled: true,
            voice_enabled: true,
            session_id: session_id.into(),
        }
    }

    /// Toggle browser mode
    pub fn toggle_mode(&mut self) {
        self.mode.toggle();
    }

    /// Set mode explicitly
    pub fn set_mode(&mut self, mode: BrowserMode) {
        self.mode = mode;
    }
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self::new("default-session")
    }
}

/// AI-Native browser controller
pub struct AiBrowser {
    /// Session configuration
    pub config: SessionConfig,
    /// Intent parser
    pub intent_parser: IntentParser,
    /// Agent connector
    pub agent: AgentConnector,
    /// Voice input handler
    pub voice: VoiceInput,
    /// Chrome UI state
    pub chrome: MinimalChrome,
    /// Navigation history for context
    pub navigation_history: Vec<String>,
}

impl AiBrowser {
    /// Create new AI browser
    pub fn new(session_id: impl Into<String>) -> Self {
        let config = SessionConfig::new(session_id);
        let intent_parser = IntentParser::new();
        let agent = AgentConnector::new();
        let voice = VoiceInput::new();
        let chrome = MinimalChrome::new();

        Self {
            config,
            intent_parser,
            agent,
            voice,
            chrome,
            navigation_history: Vec::new(),
        }
    }

    /// Process user input (could be URL or natural language)
    pub fn process_input(&mut self, input: &str) -> Action {
        // Try to parse as intent first
        if let Some(intent) = self.intent_parser.parse(input) {
            return self.execute_intent(intent);
        }

        // Fall back to treating as URL
        Action::Navigate(input.to_string())
    }

    /// Execute a parsed intent
    fn execute_intent(&mut self, intent: Intent) -> Action {
        match intent {
            Intent::Navigate { url } => {
                self.navigation_history.push(url.clone());
                Action::Navigate(url)
            }
            Intent::Search { query } => {
                Action::Search(query)
            }
            Intent::GoBack => {
                Action::Back
            }
            Intent::GoForward => {
                Action::Forward
            }
            Intent::Refresh => {
                Action::Refresh
            }
            Intent::SwitchMode { mode } => {
                self.config.set_mode(mode);
                Action::SwitchMode(mode)
            }
            Intent::AskAgent { query } => {
                Action::AgentQuery(query)
            }
            Intent::VoiceInput => {
                Action::ActivateVoice
            }
        }
    }

    /// Toggle between Traditional and AI-Native modes
    pub fn toggle_mode(&mut self) {
        self.config.toggle_mode();
    }

    /// Get current mode
    pub fn mode(&self) -> BrowserMode {
        self.config.mode
    }

    /// Enable voice input
    pub fn enable_voice(&mut self) {
        self.config.voice_enabled = true;
    }

    /// Connect to agent service
    pub async fn connect_agent(&mut self, endpoint: &str) -> anyhow::Result<()> {
        self.agent.connect(endpoint).await
    }
}

/// Actions that can result from user input
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Navigate to URL
    Navigate(String),
    /// Search with query
    Search(String),
    /// Go back in history
    Back,
    /// Go forward in history
    Forward,
    /// Refresh current page
    Refresh,
    /// Switch browser mode
    SwitchMode(BrowserMode),
    /// Query AI agent
    AgentQuery(String),
    /// Activate voice input
    ActivateVoice,
    /// No action
    None,
}

impl Action {
    /// Check if action requires navigation
    pub fn is_navigation(&self) -> bool {
        matches!(self, Action::Navigate(_) | Action::Back | Action::Forward)
    }

    /// Get URL if navigation action
    pub fn url(&self) -> Option<&str> {
        match self {
            Action::Navigate(url) => Some(url),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browser_mode_toggle() {
        let mut mode = BrowserMode::Traditional;
        mode.toggle();
        assert_eq!(mode, BrowserMode::AiNative);
        
        mode.toggle();
        assert_eq!(mode, BrowserMode::Traditional);
    }

    #[test]
    fn test_session_config() {
        let mut config = SessionConfig::new("test-session");
        assert_eq!(config.mode, BrowserMode::Traditional);
        
        config.toggle_mode();
        assert_eq!(config.mode, BrowserMode::AiNative);
        assert_eq!(config.session_id, "test-session");
    }

    #[test]
    fn test_ai_browser_creation() {
        let browser = AiBrowser::new("test");
        assert_eq!(browser.mode(), BrowserMode::Traditional);
        assert!(browser.config.ai_enabled);
    }

    #[test]
    fn test_action_navigation() {
        let action = Action::Navigate("https://example.com".to_string());
        assert!(action.is_navigation());
        assert_eq!(action.url(), Some("https://example.com"));
        
        let action = Action::Search("query".to_string());
        assert!(!action.is_navigation());
    }
}
