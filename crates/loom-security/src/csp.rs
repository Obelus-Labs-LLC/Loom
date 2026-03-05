//! Content Security Policy (CSP) Implementation
//!
//! Phase L15: Security & Sandboxing
//! - CSP header parsing (Content-Security-Policy, CSP-Report-Only)
//! - Directive enforcement (script-src, style-src, img-src, etc.)
//! - Inline script/style blocking with nonce/hash support
//! - Violation reporting

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use hashbrown::HashMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;

/// CSP Directive types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Directive {
    DefaultSrc,
    ScriptSrc,
    StyleSrc,
    ImgSrc,
    FontSrc,
    ConnectSrc,
    MediaSrc,
    ObjectSrc,
    FrameSrc,
    FrameAncestors,
    FormAction,
    BaseUri,
    Sandbox,
    UpgradeInsecureRequests,
    BlockAllMixedContent,
    ReportUri,
    ReportTo,
}

impl Directive {
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "default-src" => Some(Directive::DefaultSrc),
            "script-src" => Some(Directive::ScriptSrc),
            "style-src" => Some(Directive::StyleSrc),
            "img-src" => Some(Directive::ImgSrc),
            "font-src" => Some(Directive::FontSrc),
            "connect-src" => Some(Directive::ConnectSrc),
            "media-src" => Some(Directive::MediaSrc),
            "object-src" => Some(Directive::ObjectSrc),
            "frame-src" => Some(Directive::FrameSrc),
            "frame-ancestors" => Some(Directive::FrameAncestors),
            "form-action" => Some(Directive::FormAction),
            "base-uri" => Some(Directive::BaseUri),
            "sandbox" => Some(Directive::Sandbox),
            "upgrade-insecure-requests" => Some(Directive::UpgradeInsecureRequests),
            "block-all-mixed-content" => Some(Directive::BlockAllMixedContent),
            "report-uri" => Some(Directive::ReportUri),
            "report-to" => Some(Directive::ReportTo),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Directive::DefaultSrc => "default-src",
            Directive::ScriptSrc => "script-src",
            Directive::StyleSrc => "style-src",
            Directive::ImgSrc => "img-src",
            Directive::FontSrc => "font-src",
            Directive::ConnectSrc => "connect-src",
            Directive::MediaSrc => "media-src",
            Directive::ObjectSrc => "object-src",
            Directive::FrameSrc => "frame-src",
            Directive::FrameAncestors => "frame-ancestors",
            Directive::FormAction => "form-action",
            Directive::BaseUri => "base-uri",
            Directive::Sandbox => "sandbox",
            Directive::UpgradeInsecureRequests => "upgrade-insecure-requests",
            Directive::BlockAllMixedContent => "block-all-mixed-content",
            Directive::ReportUri => "report-uri",
            Directive::ReportTo => "report-to",
        }
    }

    pub fn uses_default_fallback(&self) -> bool {
        matches!(self,
            Directive::ScriptSrc |
            Directive::StyleSrc |
            Directive::ImgSrc |
            Directive::FontSrc |
            Directive::ConnectSrc |
            Directive::MediaSrc |
            Directive::ObjectSrc |
            Directive::FrameSrc
        )
    }
}

/// CSP Source keywords
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    Self_,
    UnsafeInline,
    UnsafeEval,
    UnsafeHashes,
    WasmUnsafeEval,
    None,
    StrictDynamic,
    ReportSample,
    Host(String),
    Scheme(String),
    Nonce(String),
    Hash { algorithm: HashAlgorithm, value: String },
    Wildcard,
}

impl Source {
    pub fn parse(expr: &str) -> Self {
        let expr = expr.trim();
        
        match expr.to_lowercase().as_str() {
            "'self'" => Source::Self_,
            "'unsafe-inline'" => Source::UnsafeInline,
            "'unsafe-eval'" => Source::UnsafeEval,
            "'unsafe-hashes'" => Source::UnsafeHashes,
            "'wasm-unsafe-eval'" => Source::WasmUnsafeEval,
            "'none'" => Source::None,
            "'strict-dynamic'" => Source::StrictDynamic,
            "'report-sample'" => Source::ReportSample,
            "*" => Source::Wildcard,
            _ => {
                if expr.to_lowercase().starts_with("'nonce-") && expr.ends_with('\'') {
                    let nonce = expr[7..expr.len()-1].to_string();
                    return Source::Nonce(nonce);
                }
                
                if expr.to_lowercase().starts_with("'sha256-") && expr.ends_with('\'') {
                    let value = expr[8..expr.len()-1].to_string();
                    return Source::Hash { algorithm: HashAlgorithm::Sha256, value };
                }
                if expr.to_lowercase().starts_with("'sha384-") && expr.ends_with('\'') {
                    let value = expr[8..expr.len()-1].to_string();
                    return Source::Hash { algorithm: HashAlgorithm::Sha384, value };
                }
                if expr.to_lowercase().starts_with("'sha512-") && expr.ends_with('\'') {
                    let value = expr[8..expr.len()-1].to_string();
                    return Source::Hash { algorithm: HashAlgorithm::Sha512, value };
                }
                
                if expr.ends_with(':') {
                    return Source::Scheme(expr.to_string());
                }
                
                Source::Host(expr.to_string())
            }
        }
    }

    pub fn allows(&self, url: &str, self_origin: &str) -> bool {
        match self {
            Source::Self_ => url.starts_with(self_origin),
            Source::Wildcard => true,
            Source::None => false,
            Source::Host(host) => url.contains(host),
            Source::Scheme(scheme) => url.starts_with(scheme),
            _ => false,
        }
    }
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Source::Self_ => write!(f, "'self'"),
            Source::UnsafeInline => write!(f, "'unsafe-inline'"),
            Source::UnsafeEval => write!(f, "'unsafe-eval'"),
            Source::UnsafeHashes => write!(f, "'unsafe-hashes'"),
            Source::WasmUnsafeEval => write!(f, "'wasm-unsafe-eval'"),
            Source::None => write!(f, "'none'"),
            Source::StrictDynamic => write!(f, "'strict-dynamic'"),
            Source::ReportSample => write!(f, "'report-sample'"),
            Source::Wildcard => write!(f, "*"),
            Source::Host(host) => write!(f, "{}", host),
            Source::Scheme(scheme) => write!(f, "{}", scheme),
            Source::Nonce(nonce) => write!(f, "'nonce-{}'", nonce),
            Source::Hash { algorithm, value } => write!(f, "'{}-{}'", algorithm.name(), value),
        }
    }
}

/// Hash algorithms for CSP
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashAlgorithm {
    Sha256,
    Sha384,
    Sha512,
}

impl HashAlgorithm {
    pub fn name(&self) -> &'static str {
        match self {
            HashAlgorithm::Sha256 => "sha256",
            HashAlgorithm::Sha384 => "sha384",
            HashAlgorithm::Sha512 => "sha512",
        }
    }
}

/// CSP Policy (parsed from header)
#[derive(Debug, Clone, Default)]
pub struct CspPolicy {
    directives: HashMap<Directive, Vec<Source>>,
    report_only: bool,
}

impl CspPolicy {
    pub fn new() -> Self {
        Self {
            directives: HashMap::new(),
            report_only: false,
        }
    }

    pub fn report_only() -> Self {
        Self {
            directives: HashMap::new(),
            report_only: true,
        }
    }

    pub fn parse(header_value: &str) -> Self {
        let mut policy = Self::new();
        
        for directive_str in header_value.split(';') {
            let directive_str = directive_str.trim();
            if directive_str.is_empty() {
                continue;
            }
            
            let parts: Vec<&str> = directive_str.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }
            
            let directive_name = parts[0];
            let sources: Vec<Source> = parts[1..]
                .iter()
                .map(|s| Source::parse(s))
                .collect();
            
            if let Some(directive) = Directive::from_name(directive_name) {
                policy.directives.insert(directive, sources);
            }
        }
        
        policy
    }

    pub fn is_report_only(&self) -> bool {
        self.report_only
    }

    pub fn set_report_only(&mut self, report_only: bool) {
        self.report_only = report_only;
    }

    pub fn get_sources(&self, directive: Directive) -> Option<&Vec<Source>> {
        if let Some(sources) = self.directives.get(&directive) {
            return Some(sources);
        }
        
        if directive.uses_default_fallback() {
            return self.directives.get(&Directive::DefaultSrc);
        }
        
        None
    }

    pub fn add_directive(&mut self, directive: Directive, sources: Vec<Source>) {
        self.directives.insert(directive, sources);
    }

    pub fn allows_inline_scripts(&self) -> bool {
        if let Some(sources) = self.get_sources(Directive::ScriptSrc) {
            sources.iter().any(|s| matches!(s, Source::UnsafeInline))
        } else {
            true
        }
    }

    pub fn allows_inline_styles(&self) -> bool {
        if let Some(sources) = self.get_sources(Directive::StyleSrc) {
            sources.iter().any(|s| matches!(s, Source::UnsafeInline))
        } else {
            true
        }
    }

    pub fn allows_eval(&self) -> bool {
        if let Some(sources) = self.get_sources(Directive::ScriptSrc) {
            sources.iter().any(|s| matches!(s, Source::UnsafeEval))
        } else {
            true
        }
    }

    pub fn is_script_nonce_valid(&self, nonce: &str) -> bool {
        if let Some(sources) = self.get_sources(Directive::ScriptSrc) {
            sources.iter().any(|s| {
                matches!(s, Source::Nonce(n) if n == nonce)
            })
        } else {
            true
        }
    }

    pub fn is_script_hash_valid(&self, hash: &str, algorithm: HashAlgorithm) -> bool {
        if let Some(sources) = self.get_sources(Directive::ScriptSrc) {
            sources.iter().any(|s| {
                matches!(s, Source::Hash { algorithm: a, value: v } if *a == algorithm && v == hash)
            })
        } else {
            true
        }
    }

    pub fn allows_url(&self, directive: Directive, url: &str, self_origin: &str) -> bool {
        if let Some(sources) = self.get_sources(directive) {
            if sources.iter().any(|s| matches!(s, Source::None)) {
                return false;
            }
            
            sources.iter().any(|s| s.allows(url, self_origin))
        } else {
            true
        }
    }

    pub fn allows_script_url(&self, url: &str, self_origin: &str) -> bool {
        self.allows_url(Directive::ScriptSrc, url, self_origin)
    }

    pub fn allows_style_url(&self, url: &str, self_origin: &str) -> bool {
        self.allows_url(Directive::StyleSrc, url, self_origin)
    }

    pub fn allows_image_url(&self, url: &str, self_origin: &str) -> bool {
        self.allows_url(Directive::ImgSrc, url, self_origin)
    }

    pub fn get_sandbox(&self) -> Option<Vec<String>> {
        self.directives.get(&Directive::Sandbox)
            .map(|sources| {
                sources.iter()
                    .filter_map(|s| match s {
                        Source::Host(h) => Some(h.clone()),
                        _ => None,
                    })
                    .collect()
            })
    }

    pub fn upgrade_insecure_requests(&self) -> bool {
        self.directives.contains_key(&Directive::UpgradeInsecureRequests)
    }

    pub fn to_header_value(&self) -> String {
        let mut parts = Vec::new();
        
        for (directive, sources) in &self.directives {
            let source_strs: Vec<String> = sources.iter()
                .map(|s| s.to_string())
                .collect();
            
            if source_strs.is_empty() {
                parts.push(directive.name().to_string());
            } else {
                parts.push(format!("{} {}", directive.name(), source_strs.join(" ")));
            }
        }
        
        parts.join("; ")
    }
}

/// CSP Enforcement Result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CspResult {
    Allow,
    Block { violation: CspViolation },
    ReportOnly { violation: CspViolation },
}

/// CSP Violation details
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CspViolation {
    pub directive: Directive,
    pub blocked_uri: String,
    pub violated_directive: String,
    pub original_policy: String,
    pub source_file: Option<String>,
    pub line_number: Option<u32>,
    pub column_number: Option<u32>,
}

/// CSP Enforcer - checks resources against policy
#[derive(Debug, Clone)]
pub struct CspEnforcer {
    policy: CspPolicy,
    self_origin: String,
}

impl CspEnforcer {
    pub fn new(policy: CspPolicy, self_origin: impl Into<String>) -> Self {
        Self {
            policy,
            self_origin: self_origin.into(),
        }
    }

    pub fn check_inline_script(&self, _script_content: &str, nonce: Option<&str>) -> CspResult {
        if let Some(nonce) = nonce {
            if self.policy.is_script_nonce_valid(nonce) {
                return CspResult::Allow;
            }
        }
        
        if self.policy.allows_inline_scripts() {
            return CspResult::Allow;
        }
        
        let violation = CspViolation {
            directive: Directive::ScriptSrc,
            blocked_uri: "inline".to_string(),
            violated_directive: self.policy.get_sources(Directive::ScriptSrc)
                .map(|s| s.iter().map(|src| src.to_string()).collect::<Vec<_>>().join(" "))
                .unwrap_or_default(),
            original_policy: self.policy.to_header_value(),
            source_file: None,
            line_number: None,
            column_number: None,
        };
        
        if self.policy.is_report_only() {
            CspResult::ReportOnly { violation }
        } else {
            CspResult::Block { violation }
        }
    }

    pub fn check_script_url(&self, url: &str) -> CspResult {
        if self.policy.allows_script_url(url, &self.self_origin) {
            return CspResult::Allow;
        }
        
        let violation = CspViolation {
            directive: Directive::ScriptSrc,
            blocked_uri: url.to_string(),
            violated_directive: self.policy.get_sources(Directive::ScriptSrc)
                .map(|s| s.iter().map(|src| src.to_string()).collect::<Vec<_>>().join(" "))
                .unwrap_or_default(),
            original_policy: self.policy.to_header_value(),
            source_file: None,
            line_number: None,
            column_number: None,
        };
        
        if self.policy.is_report_only() {
            CspResult::ReportOnly { violation }
        } else {
            CspResult::Block { violation }
        }
    }

    pub fn check_eval(&self) -> CspResult {
        if self.policy.allows_eval() {
            return CspResult::Allow;
        }
        
        let violation = CspViolation {
            directive: Directive::ScriptSrc,
            blocked_uri: "eval".to_string(),
            violated_directive: self.policy.get_sources(Directive::ScriptSrc)
                .map(|s| s.iter().map(|src| src.to_string()).collect::<Vec<_>>().join(" "))
                .unwrap_or_default(),
            original_policy: self.policy.to_header_value(),
            source_file: None,
            line_number: None,
            column_number: None,
        };
        
        if self.policy.is_report_only() {
            CspResult::ReportOnly { violation }
        } else {
            CspResult::Block { violation }
        }
    }

    pub fn policy(&self) -> &CspPolicy {
        &self.policy
    }
}

/// Parse CSP headers from HTTP response
pub fn parse_csp_headers(headers: &[(String, String)]) -> (Option<CspPolicy>, Option<CspPolicy>) {
    let mut enforce_policy: Option<CspPolicy> = None;
    let mut report_only_policy: Option<CspPolicy> = None;
    
    for (name, value) in headers {
        if name.eq_ignore_ascii_case("Content-Security-Policy") {
            enforce_policy = Some(CspPolicy::parse(value));
        } else if name.eq_ignore_ascii_case("Content-Security-Policy-Report-Only") {
            let mut policy = CspPolicy::parse(value);
            policy.set_report_only(true);
            report_only_policy = Some(policy);
        }
    }
    
    (enforce_policy, report_only_policy)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_csp_header() {
        let csp = "default-src 'self'; script-src 'self' 'unsafe-inline' https://cdn.example.com; style-src 'self' 'unsafe-inline'";
        let policy = CspPolicy::parse(csp);
        
        assert!(policy.get_sources(Directive::DefaultSrc).is_some());
        assert!(policy.get_sources(Directive::ScriptSrc).is_some());
        assert!(policy.get_sources(Directive::StyleSrc).is_some());
        assert!(policy.allows_inline_scripts());
    }

    #[test]
    fn test_csp_blocks_inline_script() {
        let csp = "script-src 'self'";
        let policy = CspPolicy::parse(csp);
        let enforcer = CspEnforcer::new(policy, "https://example.com");
        
        let result = enforcer.check_inline_script("alert('test')", None);
        assert!(matches!(result, CspResult::Block { .. }));
    }

    #[test]
    fn test_csp_allows_inline_with_unsafe_inline() {
        let csp = "script-src 'self' 'unsafe-inline'";
        let policy = CspPolicy::parse(csp);
        let enforcer = CspEnforcer::new(policy, "https://example.com");
        
        let result = enforcer.check_inline_script("alert('test')", None);
        assert_eq!(result, CspResult::Allow);
    }

    #[test]
    fn test_csp_allows_with_nonce() {
        let csp = "script-src 'self' 'nonce-abc123'";
        let policy = CspPolicy::parse(csp);
        let enforcer = CspEnforcer::new(policy, "https://example.com");
        
        let result = enforcer.check_inline_script("alert('test')", Some("abc123"));
        assert_eq!(result, CspResult::Allow);
        
        let result = enforcer.check_inline_script("alert('test')", Some("wrong"));
        assert!(matches!(result, CspResult::Block { .. }));
    }

    #[test]
    fn test_parse_source() {
        assert!(matches!(Source::parse("'self'"), Source::Self_));
        assert!(matches!(Source::parse("'unsafe-inline'"), Source::UnsafeInline));
        assert!(matches!(Source::parse("*"), Source::Wildcard));
        assert!(matches!(Source::parse("https://example.com"), Source::Host(_)));
        assert!(matches!(Source::parse("https:"), Source::Scheme(_)));
    }

    #[test]
    fn test_parse_nonce() {
        let source = Source::parse("'nonce-random123'");
        assert!(matches!(source, Source::Nonce(n) if n == "random123"));
    }

    #[test]
    fn test_parse_hash() {
        let source = Source::parse("'sha256-abc123'");
        assert!(matches!(source, Source::Hash { algorithm: HashAlgorithm::Sha256, value } if value == "abc123"));
    }

    #[test]
    fn test_default_fallback() {
        let csp = "default-src 'self'";
        let policy = CspPolicy::parse(csp);
        assert!(policy.get_sources(Directive::ScriptSrc).is_some());
    }
}
