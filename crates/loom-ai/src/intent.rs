//! Intent Parser - Natural language to browser actions
//!
//! Converts user input like "go to github.com" into structured intents

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use regex::Regex;

/// Parsed user intent
#[derive(Debug, Clone, PartialEq)]
pub enum Intent {
    /// Navigate to a specific URL
    Navigate { url: String },
    /// Perform a search
    Search { query: String },
    /// Go back in history
    GoBack,
    /// Go forward in history
    GoForward,
    /// Refresh the page
    Refresh,
    /// Switch browser mode
    SwitchMode { mode: super::BrowserMode },
    /// Ask the AI agent
    AskAgent { query: String },
    /// Activate voice input
    VoiceInput,
}

/// Intent parser for natural language input
pub struct IntentParser {
    /// Navigation patterns
    navigate_patterns: Vec<Regex>,
    /// Search patterns
    search_patterns: Vec<Regex>,
    /// History patterns
    history_patterns: Vec<Regex>,
    /// Mode switch patterns
    mode_patterns: Vec<Regex>,
    /// Agent query patterns
    agent_patterns: Vec<Regex>,
    /// Voice patterns
    voice_patterns: Vec<Regex>,
}

impl IntentParser {
    /// Create a new intent parser with default patterns
    pub fn new() -> Self {
        Self {
            navigate_patterns: vec![
                Regex::new(r"(?i)^(go to|navigate to|open|visit)\s+(.+)").unwrap(),
                Regex::new(r"(?i)^(take me to|show me|load)\s+(.+)").unwrap(),
                Regex::new(r"^(https?://|www\.)").unwrap(), // Direct URL
                Regex::new(r"(?i)^([a-zA-Z0-9-]+\.(com|org|net|io|dev|co))").unwrap(), // Domain
            ],
            search_patterns: vec![
                Regex::new(r"(?i)^(search for|search|find|look up|google)\s+(.+)").unwrap(),
                Regex::new(r"(?i)^(what is|who is|where is|how to|why is)\s+(.+)").unwrap(),
            ],
            history_patterns: vec![
                Regex::new(r"(?i)^(go back|back|previous)").unwrap(),
                Regex::new(r"(?i)^(go forward|forward|next)").unwrap(),
                Regex::new(r"(?i)^(refresh|reload|update)").unwrap(),
            ],
            mode_patterns: vec![
                Regex::new(r"(?i)^(switch to|use|enable)\s+(traditional|classic)\s+mode").unwrap(),
                Regex::new(r"(?i)^(switch to|use|enable)\s+(ai|ai-native|smart)\s+mode").unwrap(),
            ],
            agent_patterns: vec![
                Regex::new(r"(?i)^(ask|tell me|explain|help|can you)\s+(.+)").unwrap(),
                Regex::new(r"(?i)^/\\?\\s*(.+)").unwrap(), // /? prefix
            ],
            voice_patterns: vec![
                Regex::new(r"(?i)^(voice|speak|listen|microphone)").unwrap(),
                Regex::new(r"(?i)^(use voice|voice input)").unwrap(),
            ],
        }
    }

    /// Parse user input into an intent
    pub fn parse(&self, input: &str) -> Option<Intent> {
        let input = input.trim();
        
        // Check for navigation first
        if let Some(intent) = self.parse_navigate(input) {
            return Some(intent);
        }
        
        // Check for history commands
        if let Some(intent) = self.parse_history(input) {
            return Some(intent);
        }
        
        // Check for mode switch
        if let Some(intent) = self.parse_mode_switch(input) {
            return Some(intent);
        }
        
        // Check for voice activation
        if let Some(intent) = self.parse_voice(input) {
            return Some(intent);
        }
        
        // Check for agent query
        if let Some(intent) = self.parse_agent_query(input) {
            return Some(intent);
        }
        
        // Check for search
        if let Some(intent) = self.parse_search(input) {
            return Some(intent);
        }
        
        None
    }

    /// Parse navigation intent
    fn parse_navigate(&self, input: &str) -> Option<Intent> {
        // Check explicit navigation patterns
        for pattern in &self.navigate_patterns {
            if let Some(caps) = pattern.captures(input) {
                let target = caps.get(2).map(|m| m.as_str())
                    .or_else(|| caps.get(0).map(|m| m.as_str()))
                    .unwrap_or(input);
                
                let url = normalize_url(target);
                return Some(Intent::Navigate { url });
            }
        }
        
        // Check if input looks like a URL
        if looks_like_url(input) {
            return Some(Intent::Navigate { 
                url: normalize_url(input) 
            });
        }
        
        None
    }

    /// Parse search intent
    fn parse_search(&self, input: &str) -> Option<Intent> {
        for pattern in &self.search_patterns {
            if let Some(caps) = pattern.captures(input) {
                let query = caps.get(2)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_else(|| input.to_string());
                return Some(Intent::Search { query });
            }
        }
        
        None
    }

    /// Parse history navigation intent
    fn parse_history(&self, input: &str) -> Option<Intent> {
        let lower = input.to_lowercase();
        
        if self.history_patterns[0].is_match(&lower) {
            return Some(Intent::GoBack);
        }
        
        if self.history_patterns[1].is_match(&lower) {
            return Some(Intent::GoForward);
        }
        
        if self.history_patterns[2].is_match(&lower) {
            return Some(Intent::Refresh);
        }
        
        None
    }

    /// Parse mode switch intent
    fn parse_mode_switch(&self, input: &str) -> Option<Intent> {
        let lower = input.to_lowercase();
        
        if self.mode_patterns[0].is_match(&lower) {
            return Some(Intent::SwitchMode { 
                mode: super::BrowserMode::Traditional 
            });
        }
        
        if self.mode_patterns[1].is_match(&lower) {
            return Some(Intent::SwitchMode { 
                mode: super::BrowserMode::AiNative 
            });
        }
        
        None
    }

    /// Parse agent query intent
    fn parse_agent_query(&self, input: &str) -> Option<Intent> {
        for pattern in &self.agent_patterns {
            if let Some(caps) = pattern.captures(input) {
                let query = caps.get(2)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_else(|| input.to_string());
                return Some(Intent::AskAgent { query });
            }
        }
        
        None
    }

    /// Parse voice input intent
    fn parse_voice(&self, input: &str) -> Option<Intent> {
        let lower = input.to_lowercase();
        
        for pattern in &self.voice_patterns {
            if pattern.is_match(&lower) {
                return Some(Intent::VoiceInput);
            }
        }
        
        None
    }
}

impl Default for IntentParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if input looks like a URL
fn looks_like_url(input: &str) -> bool {
    input.starts_with("http://") 
        || input.starts_with("https://")
        || input.starts_with("www.")
        || input.contains('.') && !input.contains(' ')
}

/// Normalize URL (add https:// if missing)
fn normalize_url(input: &str) -> String {
    let input = input.trim();
    
    if input.starts_with("http://") || input.starts_with("https://") {
        return input.to_string();
    }
    
    if input.starts_with("www.") {
        return format!("https://{}", input);
    }
    
    // Check for domain-like patterns
    if input.contains('.') && !input.contains(' ') {
        return format!("https://{}", input);
    }
    
    input.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_navigate() {
        let parser = IntentParser::new();
        
        // Test "go to" pattern
        let intent = parser.parse("go to github.com");
        assert!(matches!(intent, Some(Intent::Navigate { url }) if url == "https://github.com"));
        
        // Test direct URL
        let intent = parser.parse("https://example.com");
        assert!(matches!(intent, Some(Intent::Navigate { url }) if url == "https://example.com"));
        
        // Test www prefix
        let intent = parser.parse("www.google.com");
        assert!(matches!(intent, Some(Intent::Navigate { url }) if url == "https://www.google.com"));
    }

    #[test]
    fn test_parse_search() {
        let parser = IntentParser::new();
        
        let intent = parser.parse("search for rust programming");
        assert!(matches!(intent, Some(Intent::Search { query }) if query == "rust programming"));
        
        let intent = parser.parse("what is web assembly");
        assert!(matches!(intent, Some(Intent::Search { query }) if query == "web assembly"));
    }

    #[test]
    fn test_parse_history() {
        let parser = IntentParser::new();
        
        let intent = parser.parse("go back");
        assert_eq!(intent, Some(Intent::GoBack));
        
        let intent = parser.parse("forward");
        assert_eq!(intent, Some(Intent::GoForward));
        
        let intent = parser.parse("refresh");
        assert_eq!(intent, Some(Intent::Refresh));
    }

    #[test]
    fn test_parse_mode_switch() {
        let parser = IntentParser::new();
        
        let intent = parser.parse("switch to traditional mode");
        assert!(matches!(intent, Some(Intent::SwitchMode { mode: super::BrowserMode::Traditional })));
        
        let intent = parser.parse("enable ai mode");
        assert!(matches!(intent, Some(Intent::SwitchMode { mode: super::BrowserMode::AiNative })));
    }

    #[test]
    fn test_parse_agent_query() {
        let parser = IntentParser::new();
        
        let intent = parser.parse("explain quantum computing");
        assert!(matches!(intent, Some(Intent::AskAgent { query }) if query == "quantum computing"));
    }

    #[test]
    fn test_normalize_url() {
        assert_eq!(normalize_url("github.com"), "https://github.com");
        assert_eq!(normalize_url("https://example.com"), "https://example.com");
        assert_eq!(normalize_url("www.test.org"), "https://www.test.org");
    }

    #[test]
    fn test_looks_like_url() {
        assert!(looks_like_url("github.com"));
        assert!(looks_like_url("https://example.com"));
        assert!(looks_like_url("www.test.com"));
        assert!(!looks_like_url("search for something"));
    }
}
