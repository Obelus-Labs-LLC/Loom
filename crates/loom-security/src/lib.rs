//! Loom Security - CSP, Sandboxing, and Permissions
//!
//! Phase L15: Security & Sandboxing
//!
//! ## Features
//! - **CSP**: Content Security Policy parsing and enforcement
//! - **Iframe Sandboxing**: sandbox attribute, X-Frame-Options, frame-ancestors
//! - **Permissions**: Camera, microphone, location, notifications UI
//! - **Referrer Policy**: Referrer stripping based on policy
//!
//! ## Example
//! ```rust
//! use loom_security::csp::{CspPolicy, CspEnforcer, CspResult};
//!
//! // Parse CSP header
//! let policy = CspPolicy::parse("script-src 'self'; style-src 'self'");
//! let enforcer = CspEnforcer::new(policy, "https://example.com");
//!
//! // Check inline script
//! match enforcer.check_inline_script("alert('test')", None) {
//!     CspResult::Allow => println!("Script allowed"),
//!     CspResult::Block { violation } => println!("Blocked: {:?}", violation),
//!     _ => {}
//! }
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod csp;
pub mod iframe;
pub mod permissions;
pub mod referrer;

// Re-export main types
pub use csp::*;
pub use iframe::*;
pub use permissions::*;
pub use referrer::*;

/// Version of the security crate
pub const VERSION: &str = "0.2.0-L15";

/// Security context for a document/frame
#[derive(Debug, Clone)]
pub struct SecurityContext {
    /// CSP enforcer
    pub csp: Option<CspEnforcer>,
    /// Iframe sandbox
    pub sandbox: IframeSandbox,
    /// Referrer policy
    pub referrer_policy: ReferrerPolicy,
    /// Origin
    pub origin: String,
    /// Is secure context (HTTPS)
    pub is_secure_context: bool,
}

impl SecurityContext {
    /// Create new security context
    pub fn new(origin: impl Into<String>) -> Self {
        let origin = origin.into();
        let is_secure_context = origin.starts_with("https://") || 
                               origin.starts_with("file://") ||
                               origin.starts_with("localhost");
        
        Self {
            csp: None,
            sandbox: IframeSandbox::unsandboxed(),
            referrer_policy: ReferrerPolicy::Empty,
            origin,
            is_secure_context,
        }
    }

    /// Set CSP policy
    pub fn with_csp(mut self, policy: CspPolicy) -> Self {
        self.csp = Some(CspEnforcer::new(policy, self.origin.clone()));
        self
    }

    /// Set sandbox
    pub fn with_sandbox(mut self, sandbox: IframeSandbox) -> Self {
        self.sandbox = sandbox;
        self
    }

    /// Set referrer policy
    pub fn with_referrer_policy(mut self, policy: ReferrerPolicy) -> Self {
        self.referrer_policy = policy;
        self
    }

    /// Check if scripts are allowed
    pub fn allows_scripts(&self) -> bool {
        let csp_allows = self.csp.as_ref()
            .map(|c| c.policy().allows_inline_scripts())
            .unwrap_or(true);
        
        let sandbox_allows = self.sandbox.allows_scripts();
        
        csp_allows && sandbox_allows
    }

    /// Check if inline script is allowed
    pub fn check_inline_script(&self, script: &str, nonce: Option<&str>) -> CspResult {
        match &self.csp {
            Some(enforcer) => enforcer.check_inline_script(script, nonce),
            None => CspResult::Allow,
        }
    }

    /// Check if script URL is allowed
    pub fn check_script_url(&self, url: &str) -> CspResult {
        match &self.csp {
            Some(enforcer) => enforcer.check_script_url(url),
            None => CspResult::Allow,
        }
    }

    /// Get effective referrer for navigation
    pub fn get_referrer(&self, target_url: &str) -> Option<String> {
        self.referrer_policy.get_referrer(&self.origin, target_url)
    }
}

/// Parse all security headers from HTTP response
#[derive(Debug, Clone, Default)]
pub struct SecurityHeaders {
    /// CSP policy (enforce)
    pub csp: Option<CspPolicy>,
    /// CSP report-only policy
    pub csp_report_only: Option<CspPolicy>,
    /// X-Frame-Options
    pub x_frame_options: Option<XFrameOptions>,
    /// Referrer policy
    pub referrer_policy: ReferrerPolicy,
}

impl SecurityHeaders {
    /// Parse headers
    pub fn parse(headers: &[(String, String)]) -> Self {
        let (csp, csp_report_only) = csp::parse_csp_headers(headers);
        
        let mut x_frame_options = None;
        for (name, value) in headers {
            if name.eq_ignore_ascii_case("X-Frame-Options") {
                x_frame_options = XFrameOptions::parse(value);
            }
        }
        
        let referrer_policy = referrer::parse_referrer_policy_header(headers);
        
        Self {
            csp,
            csp_report_only,
            x_frame_options,
            referrer_policy,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(VERSION, "0.2.0-L15");
    }

    #[test]
    fn test_security_context() {
        let ctx = SecurityContext::new("https://example.com")
            .with_csp(CspPolicy::parse("script-src 'self'"));
        
        assert!(ctx.is_secure_context);
        assert!(ctx.csp.is_some());
    }

    #[test]
    fn test_security_context_blocks_inline() {
        let ctx = SecurityContext::new("https://example.com")
            .with_csp(CspPolicy::parse("script-src 'self'")); // No 'unsafe-inline'
        
        let result = ctx.check_inline_script("alert('test')", None);
        assert!(matches!(result, CspResult::Block { .. }));
    }

    #[test]
    fn test_security_context_allows_with_unsafe_inline() {
        let ctx = SecurityContext::new("https://example.com")
            .with_csp(CspPolicy::parse("script-src 'self' 'unsafe-inline'"));
        
        assert!(ctx.allows_scripts());
        
        let result = ctx.check_inline_script("alert('test')", None);
        assert_eq!(result, CspResult::Allow);
    }

    #[test]
    fn test_sandboxed_iframe_no_js() {
        // Sandboxed iframe without allow-scripts
        let sandbox = IframeSandbox::parse("allow-same-origin");
        let ctx = SecurityContext::new("https://example.com")
            .with_sandbox(sandbox);
        
        assert!(!ctx.allows_scripts());
    }

    #[test]
    fn test_security_headers_parsing() {
        let headers = vec![
            ("Content-Security-Policy".to_string(), "script-src 'self'".to_string()),
            ("X-Frame-Options".to_string(), "DENY".to_string()),
            ("Referrer-Policy".to_string(), "no-referrer".to_string()),
        ];
        
        let security = SecurityHeaders::parse(&headers);
        assert!(security.csp.is_some());
        assert!(security.x_frame_options.is_some());
        assert_eq!(security.referrer_policy, ReferrerPolicy::NoReferrer);
    }

    #[test]
    fn test_referrer_policy() {
        let ctx = SecurityContext::new("https://example.com/page")
            .with_referrer_policy(ReferrerPolicy::Origin);
        
        let referrer = ctx.get_referrer("https://other.com/");
        assert_eq!(referrer, Some("https://example.com".to_string()));
    }
}
