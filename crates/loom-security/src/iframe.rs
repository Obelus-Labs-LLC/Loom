//! Iframe Sandboxing
//!
//! Phase L15: Security & Sandboxing
//! - sandbox attribute parsing and enforcement
//! - X-Frame-Options handling
//! - Frame-ancestors CSP directive
//! - Cross-origin isolation

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use hashbrown::HashSet;

/// Sandbox permission flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SandboxFlag {
    /// Allow form submission
    AllowForms,
    /// Allow modal windows (alert, confirm, prompt)
    AllowModals,
    /// Allow pointer lock
    AllowPointerLock,
    /// Allow popup windows
    AllowPopups,
    /// Allow presenting in fullscreen
    AllowPresentation,
    /// Allow same-origin access
    AllowSameOrigin,
    /// Allow scripts to run
    AllowScripts,
    /// Allow navigation of top-level browsing context
    AllowTopNavigation,
    /// Allow top navigation by user activation
    AllowTopNavigationByUserActivation,
    /// Allow downloads
    AllowDownloads,
}

impl SandboxFlag {
    /// Parse sandbox token
    pub fn from_token(token: &str) -> Option<Self> {
        match token.to_lowercase().as_str() {
            "allow-forms" => Some(SandboxFlag::AllowForms),
            "allow-modals" => Some(SandboxFlag::AllowModals),
            "allow-pointer-lock" => Some(SandboxFlag::AllowPointerLock),
            "allow-popups" => Some(SandboxFlag::AllowPopups),
            "allow-presentation" => Some(SandboxFlag::AllowPresentation),
            "allow-same-origin" => Some(SandboxFlag::AllowSameOrigin),
            "allow-scripts" => Some(SandboxFlag::AllowScripts),
            "allow-top-navigation" => Some(SandboxFlag::AllowTopNavigation),
            "allow-top-navigation-by-user-activation" => Some(SandboxFlag::AllowTopNavigationByUserActivation),
            "allow-downloads" => Some(SandboxFlag::AllowDownloads),
            _ => None,
        }
    }

    /// Get token string
    pub fn token(&self) -> &'static str {
        match self {
            SandboxFlag::AllowForms => "allow-forms",
            SandboxFlag::AllowModals => "allow-modals",
            SandboxFlag::AllowPointerLock => "allow-pointer-lock",
            SandboxFlag::AllowPopups => "allow-popups",
            SandboxFlag::AllowPresentation => "allow-presentation",
            SandboxFlag::AllowSameOrigin => "allow-same-origin",
            SandboxFlag::AllowScripts => "allow-scripts",
            SandboxFlag::AllowTopNavigation => "allow-top-navigation",
            SandboxFlag::AllowTopNavigationByUserActivation => "allow-top-navigation-by-user-activation",
            SandboxFlag::AllowDownloads => "allow-downloads",
        }
    }
}

/// Iframe sandbox configuration
#[derive(Debug, Clone, Default)]
pub struct IframeSandbox {
    /// Allowed permissions
    flags: HashSet<SandboxFlag>,
    /// Whether sandbox is active (empty sandbox attribute blocks all)
    is_sandboxed: bool,
}

impl IframeSandbox {
    /// Create empty sandbox (most restrictive - blocks all)
    pub fn new() -> Self {
        Self {
            flags: HashSet::new(),
            is_sandboxed: true,
        }
    }

    /// Create unsandboxed (no restrictions)
    pub fn unsandboxed() -> Self {
        Self {
            flags: HashSet::new(),
            is_sandboxed: false,
        }
    }

    /// Parse from sandbox attribute value
    pub fn parse(value: &str) -> Self {
        let mut sandbox = Self::new();
        
        for token in value.split_whitespace() {
            let token = token.trim();
            if token.is_empty() {
                continue;
            }
            
            if let Some(flag) = SandboxFlag::from_token(token) {
                sandbox.flags.insert(flag);
            }
        }
        
        sandbox
    }

    /// Add a permission
    pub fn allow(&mut self, flag: SandboxFlag) {
        self.flags.insert(flag);
    }

    /// Remove a permission
    pub fn disallow(&mut self, flag: SandboxFlag) {
        self.flags.remove(&flag);
    }

    /// Check if a permission is allowed
    pub fn is_allowed(&self, flag: SandboxFlag) -> bool {
        if !self.is_sandboxed {
            return true; // No sandbox = all allowed
        }
        self.flags.contains(&flag)
    }

    /// Check if scripts are allowed
    pub fn allows_scripts(&self) -> bool {
        self.is_allowed(SandboxFlag::AllowScripts)
    }

    /// Check if same-origin access is allowed
    pub fn allows_same_origin(&self) -> bool {
        self.is_allowed(SandboxFlag::AllowSameOrigin)
    }

    /// Check if forms can be submitted
    pub fn allows_forms(&self) -> bool {
        self.is_allowed(SandboxFlag::AllowForms)
    }

    /// Check if popups are allowed
    pub fn allows_popups(&self) -> bool {
        self.is_allowed(SandboxFlag::AllowPopups)
    }

    /// Check if top navigation is allowed
    pub fn allows_top_navigation(&self) -> bool {
        self.is_allowed(SandboxFlag::AllowTopNavigation)
    }

    /// Check if sandbox is active
    pub fn is_sandboxed(&self) -> bool {
        self.is_sandboxed
    }

    /// Serialize to attribute value
    pub fn to_attribute(&self) -> String {
        if !self.is_sandboxed {
            return String::new();
        }
        
        self.flags.iter()
            .map(|f| f.token())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// X-Frame-Options header values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XFrameOptions {
    /// DENY - page cannot be displayed in a frame
    Deny,
    /// SAMEORIGIN - page can only be displayed in a frame on the same origin
    SameOrigin,
    /// ALLOW-FROM uri - page can only be displayed in a frame on the specified origin
    AllowFrom(&'static str),
}

impl XFrameOptions {
    /// Parse header value
    pub fn parse(value: &str) -> Option<Self> {
        let value = value.trim();
        
        if value.eq_ignore_ascii_case("deny") {
            Some(XFrameOptions::Deny)
        } else if value.eq_ignore_ascii_case("sameorigin") {
            Some(XFrameOptions::SameOrigin)
        } else if value.to_lowercase().starts_with("allow-from:") {
            let origin = value[11..].trim();
            // Note: In real implementation, this would need proper lifetime management
            // For now, we only support the static variants
            None
        } else {
            None
        }
    }

    /// Check if frame is allowed
    pub fn allows_framing(&self, frame_origin: &str, page_origin: &str) -> bool {
        match self {
            XFrameOptions::Deny => false,
            XFrameOptions::SameOrigin => frame_origin == page_origin,
            XFrameOptions::AllowFrom(allowed) => frame_origin == *allowed,
        }
    }
}

/// Frame-ancestors directive values
#[derive(Debug, Clone)]
pub enum FrameAncestors {
    /// 'none' - cannot be framed
    None,
    /// 'self' - can be framed by same origin
    Self_,
    /// Specific origins allowed
    Origins(Vec<String>),
}

impl FrameAncestors {
    /// Parse directive value
    pub fn parse(value: &str) -> Self {
        let parts: Vec<&str> = value.split_whitespace().collect();
        
        if parts.is_empty() {
            return FrameAncestors::None;
        }
        
        if parts.len() == 1 && parts[0].eq_ignore_ascii_case("'none'") {
            return FrameAncestors::None;
        }
        
        if parts.len() == 1 && parts[0].eq_ignore_ascii_case("'self'") {
            return FrameAncestors::Self_;
        }
        
        let origins: Vec<String> = parts.iter()
            .filter(|p| !p.eq_ignore_ascii_case("'self'"))
            .map(|p| p.to_string())
            .collect();
        
        if origins.is_empty() {
            FrameAncestors::Self_
        } else {
            FrameAncestors::Origins(origins)
        }
    }

    /// Check if framing is allowed from origin
    pub fn allows_origin(&self, origin: &str, self_origin: &str) -> bool {
        match self {
            FrameAncestors::None => false,
            FrameAncestors::Self_ => origin == self_origin,
            FrameAncestors::Origins(origins) => {
                origins.iter().any(|o| o == origin || o == "*")
            }
        }
    }
}

/// Frame embedding check result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameCheckResult {
    /// Frame is allowed
    Allow,
    /// Frame is denied by X-Frame-Options
    DeniedByXFrameOptions,
    /// Frame is denied by CSP frame-ancestors
    DeniedByFrameAncestors,
}

/// Check if a page can be embedded in a frame
pub fn check_frame_embedding(
    frame_origin: &str,
    page_origin: &str,
    x_frame_options: Option<XFrameOptions>,
    frame_ancestors: Option<FrameAncestors>,
) -> FrameCheckResult {
    // Check X-Frame-Options first (legacy but takes precedence)
    if let Some(xfo) = x_frame_options {
        if !xfo.allows_framing(frame_origin, page_origin) {
            return FrameCheckResult::DeniedByXFrameOptions;
        }
    }
    
    // Check CSP frame-ancestors
    if let Some(fa) = frame_ancestors {
        if !fa.allows_origin(frame_origin, page_origin) {
            return FrameCheckResult::DeniedByFrameAncestors;
        }
    }
    
    FrameCheckResult::Allow
}

/// Iframe security context
#[derive(Debug, Clone)]
pub struct IframeContext {
    /// Sandbox configuration
    pub sandbox: IframeSandbox,
    /// Whether iframe is cross-origin
    pub is_cross_origin: bool,
    /// Parent origin
    pub parent_origin: String,
    /// Own origin (if same-origin)
    pub own_origin: Option<String>,
    /// CSP policy for the iframe
    pub csp_policy: Option<crate::csp::CspPolicy>,
}

impl IframeContext {
    /// Create new iframe context
    pub fn new(sandbox: IframeSandbox, parent_origin: impl Into<String>) -> Self {
        let parent_origin = parent_origin.into();
        let is_cross_origin = !sandbox.allows_same_origin();
        
        Self {
            sandbox,
            is_cross_origin,
            parent_origin,
            own_origin: None,
            csp_policy: None,
        }
    }

    /// Set the iframe's own origin
    pub fn with_origin(mut self, origin: impl Into<String>) -> Self {
        let origin = origin.into();
        self.own_origin = Some(origin.clone());
        self.is_cross_origin = !self.sandbox.allows_same_origin() || 
                               self.parent_origin != origin;
        self
    }

    /// Check if scripts can execute
    pub fn allows_scripts(&self) -> bool {
        self.sandbox.allows_scripts()
    }

    /// Check if can access parent
    pub fn can_access_parent(&self) -> bool {
        !self.is_cross_origin
    }

    /// Check if can access top
    pub fn can_access_top(&self) -> bool {
        !self.is_cross_origin || self.sandbox.allows_top_navigation()
    }

    /// Get effective origin for security checks
    pub fn effective_origin(&self) -> &str {
        if self.sandbox.allows_same_origin() {
            self.own_origin.as_deref().unwrap_or("null")
        } else {
            "null"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_flags() {
        let sandbox = IframeSandbox::parse("allow-scripts allow-same-origin");
        assert!(sandbox.allows_scripts());
        assert!(sandbox.allows_same_origin());
        assert!(!sandbox.allows_forms());
        assert!(!sandbox.allows_popups());
    }

    #[test]
    fn test_empty_sandbox() {
        let sandbox = IframeSandbox::new();
        assert!(sandbox.is_sandboxed());
        assert!(!sandbox.allows_scripts());
        assert!(!sandbox.allows_same_origin());
    }

    #[test]
    fn test_unsandboxed() {
        let sandbox = IframeSandbox::unsandboxed();
        assert!(!sandbox.is_sandboxed());
        assert!(sandbox.allows_scripts());
        assert!(sandbox.allows_same_origin());
    }

    #[test]
    fn test_x_frame_options() {
        let deny = XFrameOptions::parse("DENY").unwrap();
        assert!(!deny.allows_framing("https://evil.com", "https://example.com"));
        assert!(!deny.allows_framing("https://example.com", "https://example.com"));

        let sameorigin = XFrameOptions::parse("SAMEORIGIN").unwrap();
        assert!(!sameorigin.allows_framing("https://evil.com", "https://example.com"));
        assert!(sameorigin.allows_framing("https://example.com", "https://example.com"));
    }

    #[test]
    fn test_frame_ancestors() {
        let none = FrameAncestors::parse("'none'");
        assert!(!none.allows_origin("https://example.com", "https://example.com"));

        let self_ = FrameAncestors::parse("'self'");
        assert!(self_.allows_origin("https://example.com", "https://example.com"));
        assert!(!self_.allows_origin("https://evil.com", "https://example.com"));

        let origins = FrameAncestors::parse("https://trusted.com https://example.com");
        assert!(origins.allows_origin("https://trusted.com", "https://example.com"));
        assert!(!origins.allows_origin("https://evil.com", "https://example.com"));
    }

    #[test]
    fn test_frame_embedding_check() {
        // X-Frame-Options: DENY should block all framing
        let result = check_frame_embedding(
            "https://example.com",
            "https://example.com",
            Some(XFrameOptions::Deny),
            None,
        );
        assert_eq!(result, FrameCheckResult::DeniedByXFrameOptions);

        // CSP frame-ancestors blocking
        let result = check_frame_embedding(
            "https://evil.com",
            "https://example.com",
            None,
            Some(FrameAncestors::parse("'self'")),
        );
        assert_eq!(result, FrameCheckResult::DeniedByFrameAncestors);

        // Both allow
        let result = check_frame_embedding(
            "https://example.com",
            "https://example.com",
            Some(XFrameOptions::SameOrigin),
            Some(FrameAncestors::parse("'self'")),
        );
        assert_eq!(result, FrameCheckResult::Allow);
    }

    #[test]
    fn test_iframe_context() {
        let sandbox = IframeSandbox::parse("allow-scripts");
        let ctx = IframeContext::new(sandbox, "https://parent.com")
            .with_origin("https://child.com");

        assert!(ctx.allows_scripts());
        assert!(!ctx.can_access_parent()); // Cross-origin
        assert!(!ctx.can_access_top());
    }

    #[test]
    fn test_sandboxed_iframe_no_js() {
        // Sandboxed iframe without allow-scripts should not have JS access
        let sandbox = IframeSandbox::parse("allow-same-origin"); // No allow-scripts
        assert!(!sandbox.allows_scripts());

        let sandbox = IframeSandbox::new(); // Empty sandbox
        assert!(!sandbox.allows_scripts());
    }
}
