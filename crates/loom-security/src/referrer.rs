//! Referrer Policy handling
//!
//! Phase L15: Security & Sandboxing
//! - Referrer-Policy header parsing
//! - Referrer stripping based on policy
//! - Cross-origin referrer handling

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::string::{String, ToString};

/// Referrer Policy types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferrerPolicy {
    /// The empty string (no policy specified - browser default)
    Empty,
    /// no-referrer - Never send referrer
    NoReferrer,
    /// no-referrer-when-downgrade - Send full URL except when downgrading from HTTPS
    NoReferrerWhenDowngrade,
    /// same-origin - Send full URL for same-origin, no referrer for cross-origin
    SameOrigin,
    /// origin - Send only origin (scheme + host + port)
    Origin,
    /// strict-origin - Send only origin, no referrer when downgrading
    StrictOrigin,
    /// origin-when-cross-origin - Send full URL for same-origin, only origin for cross-origin
    OriginWhenCrossOrigin,
    /// strict-origin-when-cross-origin - Same as above, but no referrer when downgrading
    StrictOriginWhenCrossOrigin,
    /// unsafe-url - Always send full URL (least secure)
    UnsafeUrl,
}

impl Default for ReferrerPolicy {
    fn default() -> Self {
        ReferrerPolicy::Empty
    }
}

impl ReferrerPolicy {
    /// Parse policy string
    pub fn parse(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "" => ReferrerPolicy::Empty,
            "no-referrer" => ReferrerPolicy::NoReferrer,
            "no-referrer-when-downgrade" => ReferrerPolicy::NoReferrerWhenDowngrade,
            "same-origin" => ReferrerPolicy::SameOrigin,
            "origin" => ReferrerPolicy::Origin,
            "strict-origin" => ReferrerPolicy::StrictOrigin,
            "origin-when-cross-origin" => ReferrerPolicy::OriginWhenCrossOrigin,
            "strict-origin-when-cross-origin" => ReferrerPolicy::StrictOriginWhenCrossOrigin,
            "unsafe-url" => ReferrerPolicy::UnsafeUrl,
            _ => ReferrerPolicy::Empty,
        }
    }

    /// Get policy name
    pub fn name(&self) -> &'static str {
        match self {
            ReferrerPolicy::Empty => "",
            ReferrerPolicy::NoReferrer => "no-referrer",
            ReferrerPolicy::NoReferrerWhenDowngrade => "no-referrer-when-downgrade",
            ReferrerPolicy::SameOrigin => "same-origin",
            ReferrerPolicy::Origin => "origin",
            ReferrerPolicy::StrictOrigin => "strict-origin",
            ReferrerPolicy::OriginWhenCrossOrigin => "origin-when-cross-origin",
            ReferrerPolicy::StrictOriginWhenCrossOrigin => "strict-origin-when-cross-origin",
            ReferrerPolicy::UnsafeUrl => "unsafe-url",
        }
    }

    /// Determine referrer to send
    pub fn get_referrer(&self, source_url: &str, target_url: &str) -> Option<String> {
        let source_origin = extract_origin(source_url)?;
        let target_origin = extract_origin(target_url)?;
        
        let is_same_origin = source_origin == target_origin;
        let is_downgrade = is_https(source_url) && !is_https(target_url);
        
        match self {
            ReferrerPolicy::Empty | ReferrerPolicy::NoReferrerWhenDowngrade => {
                // Default behavior: no referrer when downgrading
                if is_downgrade {
                    None
                } else {
                    Some(source_url.to_string())
                }
            }
            ReferrerPolicy::NoReferrer => None,
            ReferrerPolicy::SameOrigin => {
                if is_same_origin {
                    Some(source_url.to_string())
                } else {
                    None
                }
            }
            ReferrerPolicy::Origin => Some(source_origin),
            ReferrerPolicy::StrictOrigin => {
                if is_downgrade {
                    None
                } else {
                    Some(source_origin)
                }
            }
            ReferrerPolicy::OriginWhenCrossOrigin => {
                if is_same_origin {
                    Some(source_url.to_string())
                } else {
                    Some(source_origin)
                }
            }
            ReferrerPolicy::StrictOriginWhenCrossOrigin => {
                if is_downgrade {
                    None
                } else if is_same_origin {
                    Some(source_url.to_string())
                } else {
                    Some(source_origin)
                }
            }
            ReferrerPolicy::UnsafeUrl => Some(source_url.to_string()),
        }
    }
}

/// Extract origin from URL (scheme://host:port)
fn extract_origin(url: &str) -> Option<String> {
    // Simple origin extraction
    // In production, use proper URL parsing
    
    if let Some(after_scheme) = url.split("://").nth(1) {
        // Get just the host:port part
        let origin_part = after_scheme.split('/').next()?;
        let scheme = url.split("://").next()?;
        Some(format!("{}://{}", scheme, origin_part))
    } else {
        None
    }
}

/// Check if URL is HTTPS
fn is_https(url: &str) -> bool {
    url.to_lowercase().starts_with("https://")
}

/// Referrer stripping utility
pub struct ReferrerStripper {
    policy: ReferrerPolicy,
}

impl ReferrerStripper {
    /// Create new stripper with policy
    pub fn new(policy: ReferrerPolicy) -> Self {
        Self { policy }
    }

    /// Strip referrer according to policy
    pub fn strip(&self, referrer: &str, destination: &str) -> Option<String> {
        self.policy.get_referrer(referrer, destination)
    }
}

/// Parse Referrer-Policy header from HTTP response
pub fn parse_referrer_policy_header(headers: &[(String, String)]) -> ReferrerPolicy {
    for (name, value) in headers {
        if name.eq_ignore_ascii_case("Referrer-Policy") {
            // Take first policy in the list
            let first_policy = value.split(',').next().unwrap_or("").trim();
            return ReferrerPolicy::parse(first_policy);
        }
    }
    
    ReferrerPolicy::Empty
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_referrer_policy() {
        assert_eq!(ReferrerPolicy::parse("no-referrer"), ReferrerPolicy::NoReferrer);
        assert_eq!(ReferrerPolicy::parse("origin"), ReferrerPolicy::Origin);
        assert_eq!(ReferrerPolicy::parse("same-origin"), ReferrerPolicy::SameOrigin);
        assert_eq!(ReferrerPolicy::parse(""), ReferrerPolicy::Empty);
    }

    #[test]
    fn test_no_referrer() {
        let policy = ReferrerPolicy::NoReferrer;
        assert_eq!(policy.get_referrer("https://example.com/page", "https://other.com/"), None);
    }

    #[test]
    fn test_origin_policy() {
        let policy = ReferrerPolicy::Origin;
        assert_eq!(
            policy.get_referrer("https://example.com/page", "https://other.com/"),
            Some("https://example.com".to_string())
        );
    }

    #[test]
    fn test_same_origin_policy() {
        let policy = ReferrerPolicy::SameOrigin;
        
        // Same origin: full URL
        assert_eq!(
            policy.get_referrer("https://example.com/page1", "https://example.com/page2"),
            Some("https://example.com/page1".to_string())
        );
        
        // Cross-origin: no referrer
        assert_eq!(
            policy.get_referrer("https://example.com/page", "https://other.com/"),
            None
        );
    }

    #[test]
    fn test_origin_when_cross_origin() {
        let policy = ReferrerPolicy::OriginWhenCrossOrigin;
        
        // Same origin: full URL
        assert_eq!(
            policy.get_referrer("https://example.com/page1", "https://example.com/page2"),
            Some("https://example.com/page1".to_string())
        );
        
        // Cross-origin: only origin
        assert_eq!(
            policy.get_referrer("https://example.com/page", "https://other.com/"),
            Some("https://example.com".to_string())
        );
    }

    #[test]
    fn test_strict_origin_when_downgrade() {
        let policy = ReferrerPolicy::StrictOriginWhenCrossOrigin;
        
        // HTTPS to HTTP downgrade: no referrer
        assert_eq!(
            policy.get_referrer("https://example.com/page", "http://other.com/"),
            None
        );
    }

    #[test]
    fn test_unsafe_url() {
        let policy = ReferrerPolicy::UnsafeUrl;
        
        // Always full URL
        assert_eq!(
            policy.get_referrer("https://example.com/sensitive", "https://other.com/"),
            Some("https://example.com/sensitive".to_string())
        );
    }

    #[test]
    fn test_extract_origin() {
        assert_eq!(
            extract_origin("https://example.com:443/page"),
            Some("https://example.com:443".to_string())
        );
        assert_eq!(
            extract_origin("http://example.com/path"),
            Some("http://example.com".to_string())
        );
    }
}
