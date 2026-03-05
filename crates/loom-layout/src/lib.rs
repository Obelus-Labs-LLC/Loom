//! Loom Layout Engine - CSS layout implementation
//!
//! Phase L11: Links and Navigation
//! - CSS parsing and style computation
//! - Block layout, flexbox, positioning
//! - Box tree construction
//! - Layout pass and paint preparation
//! - Hit-testing and navigation

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod css_types;
pub mod style_engine;
pub mod layout_engine;
pub mod dom;
pub mod style;
pub mod navigation;
pub mod hittest;

// Re-export main types
pub use css_types::*;
pub use style_engine::*;
pub use layout_engine::*;
pub use dom::*;
pub use navigation::*;
pub use hittest::*;

/// Version info
pub const VERSION: &str = "0.1.0-L9";

/// Build layout tree from HTML document and CSS
pub fn build_and_layout(html: &str, css: Option<&str>, viewport_width: f32, viewport_height: f32) -> LayoutNode {
    use alloc::string::ToString;
    
    // Parse CSS
    let mut cascade = CascadeContext::new();
    if let Some(css_text) = css {
        cascade.author = CssStylesheet::parse(css_text);
    }
    
    // Create root element
    let root_element = ElementData::new("html");
    let root_style = cascade.compute_style(&root_element, None);
    
    // Build layout tree
    let mut root = LayoutNode::new(root_style);
    
    // Parse HTML and build tree (simplified)
    // In a full implementation, this would parse the HTML and create the full tree
    
    // Layout
    let ctx = LayoutContext::new(viewport_width, viewport_height);
    layout_tree(&mut root, &ctx);
    
    root
}

/// Simple layout test
pub fn test_layout() {
    let html = r#"<html><body><div>Hello</div></body></html>"#;
    let css = r#"
        body { 
            display: flex; 
            flex-direction: column;
        }
        div { 
            width: 200px; 
            height: 100px; 
            background-color: #FF0000;
            margin: 10px;
        }
    "#;
    
    let root = build_and_layout(html, Some(css), 800.0, 600.0);
    
    // Check layout
    assert!(root.box_.content_width > 0.0);
    assert!(root.box_.content_height > 0.0);
}