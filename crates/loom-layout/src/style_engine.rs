//! Style computation engine: cascade, specificity, inheritance

use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::string::ToString;
use core::cmp::Ordering;

use crate::css_types::*;
use crate::dom::ElementData;

/// CSS selector specificity: (a, b, c) where
/// a = ID selectors
/// b = class selectors, attribute selectors, pseudo-classes
/// c = type selectors, pseudo-elements
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Specificity(pub u32, pub u32, pub u32);

impl Specificity {
    pub const ZERO: Self = Self(0, 0, 0);
    pub const INLINE: Self = Self(1000, 0, 0); // Inline styles have highest specificity
    
    /// Calculate specificity for a selector string
    pub fn from_selector(selector: &str) -> Self {
        let mut a = 0;
        let mut b = 0;
        let mut c = 0;
        
        for part in selector.split_whitespace() {
            // Count IDs
            if part.starts_with('#') {
                a += 1;
            }
            // Count classes and attributes
            else if part.starts_with('.') || part.starts_with('[') || part.starts_with(':') {
                b += 1;
            }
            // Count elements
            else if !part.is_empty() && !part.starts_with('*') {
                c += 1;
            }
        }
        
        Self(a, b, c)
    }
}

impl Ord for Specificity {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.0, self.1, self.2).cmp(&(other.0, other.1, other.2))
    }
}

impl PartialOrd for Specificity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// A CSS rule with selector and computed specificity
#[derive(Debug, Clone)]
pub struct StyleRule {
    pub selector: String,
    pub specificity: Specificity,
    pub properties: BTreeMap<String, String>, // property name -> value
}

impl StyleRule {
    pub fn new(selector: &str) -> Self {
        Self {
            selector: selector.to_string(),
            specificity: Specificity::from_selector(selector),
            properties: BTreeMap::new(),
        }
    }
    
    pub fn set_property(&mut self, name: &str, value: &str) {
        self.properties.insert(name.to_string(), value.to_string());
    }
}

/// Stylesheet containing all rules
#[derive(Debug, Default)]
pub struct CssStylesheet {
    pub rules: Vec<StyleRule>,
}

impl CssStylesheet {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }
    
    pub fn add_rule(&mut self, rule: StyleRule) {
        self.rules.push(rule);
    }
    
    /// Parse simple CSS text (selector { prop: value; ... })
    pub fn parse(css: &str) -> Self {
        let mut sheet = Self::new();
        let mut current_rule: Option<StyleRule> = None;
        
        for line in css.lines() {
            let line = line.trim();
            
            if line.is_empty() || line.starts_with("/*") {
                continue;
            }
            
            // Start of rule
            if line.contains('{') {
                let selector = line.split('{').next().unwrap_or("").trim();
                if !selector.is_empty() {
                    current_rule = Some(StyleRule::new(selector));
                }
            }
            // Property declaration
            else if let Some(rule) = current_rule.as_mut() {
                if line.contains(':') && line.contains(';') {
                    let parts: Vec<&str> = line.splitn(2, ':').collect();
                    if parts.len() == 2 {
                        let prop = parts[0].trim();
                        let value = parts[1].trim().trim_end_matches(';').trim();
                        rule.set_property(prop, value);
                    }
                }
            }
            // End of rule
            if line.contains('}') {
                if let Some(rule) = current_rule.take() {
                    sheet.add_rule(rule);
                }
            }
        }
        
        sheet
    }
}

/// Selector matching extension for ElementData
pub trait SelectorMatch {
    fn matches_selector(&self, selector: &str) -> bool;
}

impl SelectorMatch for ElementData {
    /// Check if element matches a selector
    fn matches_selector(&self, selector: &str) -> bool {
        let selector = selector.trim();
        
        // Universal selector
        if selector == "*" {
            return true;
        }
        
        // ID selector
        if selector.starts_with('#') {
            let id = &selector[1..];
            return self.id.as_ref().map(|s| s == id).unwrap_or(false);
        }
        
        // Class selector
        if selector.starts_with('.') {
            let class = &selector[1..];
            return self.classes.contains(&class.to_string());
        }
        
        // Tag selector (simple)
        if !selector.contains('.') && !selector.contains('#') && !selector.contains('[') {
            return self.tag_name.eq_ignore_ascii_case(selector);
        }
        
        // Compound selector (e.g., "div.class#id")
        let mut parts = selector.split(|c| c == '.' || c == '#' || c == '[');
        let tag_part = parts.next().unwrap_or("").trim();
        
        // Check tag if present
        if !tag_part.is_empty() && !self.tag_name.eq_ignore_ascii_case(tag_part) {
            return false;
        }
        
        // Check remaining parts
        let remainder: String = selector.chars()
            .skip(tag_part.len())
            .collect();
        
        for part in remainder.split(|c: char| c == '.' || c == '#') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            
            // Check if it's an ID or class
            if selector.contains(&format!("# {}", part).replace(' ', "")) {
                if self.id.as_ref() != Some(&part.to_string()) {
                    return false;
                }
            } else if selector.contains(&format!(". {}", part).replace(' ', "")) {
                if !self.classes.contains(&part.to_string()) {
                    return false;
                }
            }
        }
        
        true
    }
}

/// Style cascade context
pub struct CascadeContext {
    pub user_agent: CssStylesheet,
    pub author: CssStylesheet,
}

impl CascadeContext {
    pub fn new() -> Self {
        Self {
            user_agent: default_user_agent_styles(),
            author: CssStylesheet::new(),
        }
    }
    
    /// Compute final style for an element
    pub fn compute_style(&self, element: &ElementData, inline_styles: Option<&BTreeMap<String, String>>) -> ComputedStyle {
        let mut declarations: Vec<(Specificity, &String, &String)> = Vec::new();
        
        // Collect from user agent stylesheet
        for rule in &self.user_agent.rules {
            if element.matches_selector(&rule.selector) {
                for (prop, value) in &rule.properties {
                    declarations.push((rule.specificity, prop, value));
                }
            }
        }
        
        // Collect from author stylesheet
        for rule in &self.author.rules {
            if element.matches_selector(&rule.selector) {
                for (prop, value) in &rule.properties {
                    declarations.push((rule.specificity, prop, value));
                }
            }
        }
        
        // Collect inline styles (highest specificity)
        if let Some(inline) = inline_styles {
            for (prop, value) in inline {
                declarations.push((Specificity::INLINE, prop, value));
            }
        }
        
        // Sort by specificity (higher wins)
        declarations.sort_by(|a, b| a.0.cmp(&b.0));
        
        // Build computed style from winning declarations
        let mut computed = ComputedStyle::default();
        
        for (_, prop, value) in declarations {
            apply_property(&mut computed, prop, value);
        }
        
        // Apply inheritance
        // (In a real implementation, we'd inherit from parent computed styles)
        
        computed
    }
}

/// Apply a CSS property to computed style
fn apply_property(style: &mut ComputedStyle, prop: &str, value: &str) {
    let prop = prop.trim();
    let value = value.trim();
    
    match prop {
        // Display
        "display" => style.display = parse_display(value),
        
        // Position
        "position" => style.position = parse_position(value),
        "top" => style.top = parse_length(value),
        "right" => style.right = parse_length(value),
        "bottom" => style.bottom = parse_length(value),
        "left" => style.left = parse_length(value),
        "z-index" => {
            if let Ok(z) = value.parse::<i32>() {
                style.z_index = Some(z);
            }
        }
        
        // Box dimensions
        "width" => style.width = parse_length(value),
        "height" => style.height = parse_length(value),
        "min-width" => style.min_width = parse_length(value),
        "min-height" => style.min_height = parse_length(value),
        "max-width" => style.max_width = parse_length(value),
        "max-height" => style.max_height = parse_length(value),
        
        // Margin
        "margin" => {
            let lengths: Vec<_> = value.split_whitespace().map(parse_length).collect();
            match lengths.len() {
                1 => {
                    style.margin_top = lengths[0];
                    style.margin_right = lengths[0];
                    style.margin_bottom = lengths[0];
                    style.margin_left = lengths[0];
                }
                2 => {
                    style.margin_top = lengths[0];
                    style.margin_bottom = lengths[0];
                    style.margin_right = lengths[1];
                    style.margin_left = lengths[1];
                }
                4 => {
                    style.margin_top = lengths[0];
                    style.margin_right = lengths[1];
                    style.margin_bottom = lengths[2];
                    style.margin_left = lengths[3];
                }
                _ => {}
            }
        }
        "margin-top" => style.margin_top = parse_length(value),
        "margin-right" => style.margin_right = parse_length(value),
        "margin-bottom" => style.margin_bottom = parse_length(value),
        "margin-left" => style.margin_left = parse_length(value),
        
        // Padding
        "padding" => {
            let lengths: Vec<_> = value.split_whitespace().map(parse_length).collect();
            match lengths.len() {
                1 => {
                    style.padding_top = lengths[0];
                    style.padding_right = lengths[0];
                    style.padding_bottom = lengths[0];
                    style.padding_left = lengths[0];
                }
                2 => {
                    style.padding_top = lengths[0];
                    style.padding_bottom = lengths[0];
                    style.padding_right = lengths[1];
                    style.padding_left = lengths[1];
                }
                4 => {
                    style.padding_top = lengths[0];
                    style.padding_right = lengths[1];
                    style.padding_bottom = lengths[2];
                    style.padding_left = lengths[3];
                }
                _ => {}
            }
        }
        "padding-top" => style.padding_top = parse_length(value),
        "padding-right" => style.padding_right = parse_length(value),
        "padding-bottom" => style.padding_bottom = parse_length(value),
        "padding-left" => style.padding_left = parse_length(value),
        
        // Borders
        "border" => {
            // Parse border shorthand (width style color)
            let parts: Vec<_> = value.split_whitespace().collect();
            for part in parts {
                if let Ok(w) = part.parse::<f32>() {
                    style.border_top_width = w;
                    style.border_right_width = w;
                    style.border_bottom_width = w;
                    style.border_left_width = w;
                } else {
                    let bs = parse_border_style(part);
                    if bs != BorderStyle::None || part == "none" {
                        style.border_top_style = bs;
                        style.border_right_style = bs;
                        style.border_bottom_style = bs;
                        style.border_left_style = bs;
                    }
                }
            }
        }
        "border-width" => {
            if let Ok(w) = value.parse::<f32>() {
                style.border_top_width = w;
                style.border_right_width = w;
                style.border_bottom_width = w;
                style.border_left_width = w;
            }
        }
        "border-style" => {
            let bs = parse_border_style(value);
            style.border_top_style = bs;
            style.border_right_style = bs;
            style.border_bottom_style = bs;
            style.border_left_style = bs;
        }
        "border-color" => {
            if let Some(c) = Color::parse_hex(value) {
                style.border_top_color = c;
                style.border_right_color = c;
                style.border_bottom_color = c;
                style.border_left_color = c;
            }
        }
        
        // Box sizing
        "box-sizing" => style.box_sizing = parse_box_sizing(value),
        
        // Flexbox
        "flex-direction" => style.flex_direction = parse_flex_direction(value),
        "flex-wrap" => style.flex_wrap = if value == "wrap" { FlexWrap::Wrap } else { FlexWrap::Nowrap },
        "justify-content" => style.justify_content = parse_justify_content(value),
        "align-items" => style.align_items = parse_align_items(value),
        "align-self" => style.align_self = match value {
            "flex-start" => AlignSelf::FlexStart,
            "flex-end" => AlignSelf::FlexEnd,
            "center" => AlignSelf::Center,
            "stretch" => AlignSelf::Stretch,
            _ => AlignSelf::Auto,
        },
        "flex-grow" => {
            if let Ok(g) = value.parse::<f32>() {
                style.flex_grow = g;
            }
        }
        "flex-shrink" => {
            if let Ok(s) = value.parse::<f32>() {
                style.flex_shrink = s;
            }
        }
        "flex-basis" => style.flex_basis = parse_length(value),
        "order" => {
            if let Ok(o) = value.parse::<i32>() {
                style.order = o;
            }
        }
        "gap" => style.gap = parse_length(value),
        
        // Visual
        "background-color" => {
            if let Some(c) = Color::parse_hex(value) {
                style.background_color = c;
            }
        }
        "color" => {
            if let Some(c) = Color::parse_hex(value) {
                style.color = c;
            }
        }
        "opacity" => {
            if let Ok(o) = value.parse::<f32>() {
                style.opacity = o.clamp(0.0, 1.0);
            }
        }
        
        // Text
        "font-size" => style.font_size = parse_length(value),
        "font-weight" => {
            if let Ok(w) = value.parse::<u16>() {
                style.font_weight = w;
            }
        }
        "line-height" => style.line_height = parse_length(value),
        "text-align" => style.text_align = parse_text_align(value),
        
        _ => {}
    }
}

/// Default user agent styles
fn default_user_agent_styles() -> CssStylesheet {
    let mut sheet = CssStylesheet::new();
    
    // html, body
    let mut html = StyleRule::new("html, body");
    html.set_property("display", "block");
    html.set_property("margin", "0");
    html.set_property("padding", "0");
    sheet.add_rule(html);
    
    // Block elements
    for tag in &["div", "p", "h1", "h2", "h3", "h4", "h5", "h6", 
                 "header", "footer", "section", "article", "nav", "aside",
                 "main", "address", "blockquote", "pre", "ul", "ol", "li"] {
        let mut rule = StyleRule::new(tag);
        rule.set_property("display", "block");
        sheet.add_rule(rule);
    }
    
    // Headings
    let mut h1 = StyleRule::new("h1");
    h1.set_property("font-size", "2em");
    h1.set_property("font-weight", "bold");
    h1.set_property("margin", "0.67em 0");
    sheet.add_rule(h1);
    
    let mut h2 = StyleRule::new("h2");
    h2.set_property("font-size", "1.5em");
    h2.set_property("font-weight", "bold");
    h2.set_property("margin", "0.75em 0");
    sheet.add_rule(h2);
    
    let mut h3 = StyleRule::new("h3");
    h3.set_property("font-size", "1.17em");
    h3.set_property("font-weight", "bold");
    h3.set_property("margin", "0.83em 0");
    sheet.add_rule(h3);
    
    // Paragraph
    let mut p = StyleRule::new("p");
    p.set_property("margin", "1em 0");
    sheet.add_rule(p);
    
    // Inline elements
    for tag in &["span", "a", "em", "i", "strong", "b", "code", "small"] {
        let mut rule = StyleRule::new(tag);
        rule.set_property("display", "inline");
        sheet.add_rule(rule);
    }
    
    // Links
    let mut a = StyleRule::new("a");
    a.set_property("color", "#0000EE");
    a.set_property("text-decoration", "underline");
    sheet.add_rule(a);
    
    // Hidden elements
    for tag in &["script", "style", "meta", "link", "title"] {
        let mut rule = StyleRule::new(tag);
        rule.set_property("display", "none");
        sheet.add_rule(rule);
    }
    
    sheet
}