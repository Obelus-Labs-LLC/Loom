//! Minimal Chrome - AI-Native UI
//!
//! Address bar only, no toolbar, clean minimal interface

use alloc::string::String;

/// Minimal chrome UI for AI-Native mode
#[derive(Debug, Clone, Default)]
pub struct MinimalChrome {
    /// Address bar content
    pub address_bar: String,
    /// Address bar focused
    pub address_focused: bool,
    /// Show suggestions dropdown
    pub show_suggestions: bool,
    /// Suggestions list
    pub suggestions: alloc::vec::Vec<String>,
    /// Loading indicator
    pub is_loading: bool,
    /// Page title
    pub page_title: String,
    /// Security indicator (secure, insecure, etc.)
    pub security_indicator: SecurityIndicator,
}

/// Security indicator for address bar
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SecurityIndicator {
    /// Secure HTTPS connection
    Secure,
    /// Insecure HTTP connection
    Insecure,
    /// Mixed content warnings
    Mixed,
    /// Invalid certificate
    Invalid,
    /// Local file
    Local,
    /// Internal page
    #[default]
    None,
}

impl SecurityIndicator {
    /// Get icon for indicator
    pub fn icon(&self) -> &'static str {
        match self {
            SecurityIndicator::Secure => "🔒",
            SecurityIndicator::Insecure => "⚠️",
            SecurityIndicator::Mixed => "⚠️",
            SecurityIndicator::Invalid => "🚫",
            SecurityIndicator::Local => "📁",
            SecurityIndicator::None => "",
        }
    }

    /// Get color as hex
    pub fn color(&self) -> u32 {
        match self {
            SecurityIndicator::Secure => 0xFF4CAF50,      // Green
            SecurityIndicator::Insecure => 0xFFFFA726,    // Orange
            SecurityIndicator::Mixed => 0xFFFFA726,       // Orange
            SecurityIndicator::Invalid => 0xFFE53935,     // Red
            SecurityIndicator::Local => 0xFF42A5F5,       // Blue
            SecurityIndicator::None => 0xFF9E9E9E,        // Gray
        }
    }
}

impl MinimalChrome {
    /// Create new minimal chrome
    pub fn new() -> Self {
        Self {
            address_bar: String::new(),
            address_focused: false,
            show_suggestions: false,
            suggestions: alloc::vec::Vec::new(),
            is_loading: false,
            page_title: String::new(),
            security_indicator: SecurityIndicator::None,
        }
    }

    /// Set address bar content
    pub fn set_address(&mut self, url: impl Into<String>) {
        self.address_bar = url.into();
    }

    /// Get address bar content
    pub fn address(&self) -> &str {
        &self.address_bar
    }

    /// Focus the address bar
    pub fn focus_address(&mut self) {
        self.address_focused = true;
        self.show_suggestions = true;
    }

    /// Blur the address bar
    pub fn blur_address(&mut self) {
        self.address_focused = false;
        self.show_suggestions = false;
    }

    /// Set loading state
    pub fn set_loading(&mut self, loading: bool) {
        self.is_loading = loading;
    }

    /// Set page title
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.page_title = title.into();
    }

    /// Set security indicator
    pub fn set_security(&mut self, indicator: SecurityIndicator) {
        self.security_indicator = indicator;
    }

    /// Update suggestions based on input
    pub fn update_suggestions(&mut self, input: &str, history: &[String]) {
        self.suggestions.clear();
        
        if input.is_empty() {
            return;
        }
        
        let input_lower = input.to_lowercase();
        
        // Filter history that matches input
        for url in history {
            if url.to_lowercase().contains(&input_lower) {
                self.suggestions.push(url.clone());
            }
            
            if self.suggestions.len() >= 5 {
                break;
            }
        }
        
        self.show_suggestions = !self.suggestions.is_empty();
    }

    /// Clear suggestions
    pub fn clear_suggestions(&mut self) {
        self.suggestions.clear();
        self.show_suggestions = false;
    }

    /// Get display text for address bar
    /// Shows title if focused, URL otherwise
    pub fn display_text(&self) -> &str {
        if self.address_focused && !self.page_title.is_empty() {
            &self.page_title
        } else {
            &self.address_bar
        }
    }
}

/// Layout calculations for minimal chrome
pub struct ChromeLayout {
    /// Address bar position (x, y, width, height)
    pub address_bar_rect: (u32, u32, u32, u32),
    /// Suggestions dropdown position
    pub suggestions_rect: (u32, u32, u32, u32),
    /// Loading indicator position
    pub loading_rect: (u32, u32, u32, u32),
}

impl ChromeLayout {
    /// Calculate layout for given window size
    pub fn calculate(window_width: u32, window_height: u32) -> Self {
        let address_height = 40u32;
        let padding = 16u32;
        let address_width = window_width.saturating_sub(padding * 2);
        
        Self {
            address_bar_rect: (padding, padding, address_width, address_height),
            suggestions_rect: (padding, padding + address_height, address_width, 200),
            loading_rect: (window_width - 40, padding + 10, 20, 20),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_chrome() {
        let mut chrome = MinimalChrome::new();
        
        chrome.set_address("https://github.com");
        assert_eq!(chrome.address(), "https://github.com");
        
        chrome.focus_address();
        assert!(chrome.address_focused);
        
        chrome.set_title("GitHub");
        assert_eq!(chrome.display_text(), "GitHub");
    }

    #[test]
    fn test_security_indicator() {
        let secure = SecurityIndicator::Secure;
        assert_eq!(secure.icon(), "🔒");
        assert_eq!(secure.color(), 0xFF4CAF50);
        
        let insecure = SecurityIndicator::Insecure;
        assert_eq!(insecure.icon(), "⚠️");
    }

    #[test]
    fn test_suggestions() {
        let mut chrome = MinimalChrome::new();
        let history = alloc::vec![
            "https://github.com".to_string(),
            "https://google.com".to_string(),
            "https://gitlab.com".to_string(),
        ];
        
        chrome.update_suggestions("git", &history);
        assert_eq!(chrome.suggestions.len(), 2);
        assert!(chrome.show_suggestions);
    }

    #[test]
    fn test_chrome_layout() {
        let layout = ChromeLayout::calculate(1024, 768);
        
        assert_eq!(layout.address_bar_rect.0, 16); // x
        assert_eq!(layout.address_bar_rect.1, 16); // y
        assert_eq!(layout.address_bar_rect.2, 992); // width
        assert_eq!(layout.address_bar_rect.3, 40); // height
    }
}
