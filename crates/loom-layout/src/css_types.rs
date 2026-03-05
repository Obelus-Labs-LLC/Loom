//! CSS type definitions for layout engine

use alloc::string::String;
use alloc::vec::Vec;

/// CSS display property
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Display {
    Block,
    Inline,
    InlineBlock,
    Flex,
    Grid,
    None,
    Table,
    TableRow,
    TableCell,
}

impl Default for Display {
    fn default() -> Self { Display::Inline }
}

/// CSS position property
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Position {
    Static,
    Relative,
    Absolute,
    Fixed,
    Sticky,
}

impl Default for Position {
    fn default() -> Self { Position::Static }
}

/// CSS flex direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexDirection {
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

impl Default for FlexDirection {
    fn default() -> Self { FlexDirection::Row }
}

/// CSS flex wrap
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexWrap {
    Nowrap,
    Wrap,
    WrapReverse,
}

impl Default for FlexWrap {
    fn default() -> Self { FlexWrap::Nowrap }
}

/// CSS justify-content (main axis alignment)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JustifyContent {
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

impl Default for JustifyContent {
    fn default() -> Self { JustifyContent::FlexStart }
}

/// CSS align-items (cross axis alignment for container)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignItems {
    FlexStart,
    FlexEnd,
    Center,
    Stretch,
    Baseline,
}

impl Default for AlignItems {
    fn default() -> Self { AlignItems::Stretch }
}

/// CSS align-self (cross axis alignment for item)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignSelf {
    Auto,
    FlexStart,
    FlexEnd,
    Center,
    Stretch,
    Baseline,
}

impl Default for AlignSelf {
    fn default() -> Self { AlignSelf::Auto }
}

/// CSS box sizing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoxSizing {
    ContentBox,
    BorderBox,
}

impl Default for BoxSizing {
    fn default() -> Self { BoxSizing::ContentBox }
}

/// CSS length value
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Length {
    Px(f32),
    Percent(f32),
    Em(f32),
    Rem(f32),
    Auto,
    Zero,
}

impl Default for Length {
    fn default() -> Self { Length::Auto }
}

impl Length {
    /// Resolve length to pixels given context
    pub fn to_px(&self, container_size: f32, font_size: f32) -> f32 {
        match self {
            Length::Px(v) => *v,
            Length::Percent(p) => container_size * (p / 100.0),
            Length::Em(v) => font_size * v,
            Length::Rem(v) => 16.0 * v, // Default root font size
            Length::Auto => 0.0, // Must be handled specially
            Length::Zero => 0.0,
        }
    }
    
    pub fn is_auto(&self) -> bool {
        matches!(self, Length::Auto)
    }
}

/// CSS color value
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const BLACK: Self = Self { r: 0, g: 0, b: 0, a: 255 };
    pub const WHITE: Self = Self { r: 255, g: 255, b: 255, a: 255 };
    pub const TRANSPARENT: Self = Self { r: 0, g: 0, b: 0, a: 0 };
    
    /// Parse hex color (#RGB, #RGBA, #RRGGBB, #RRGGBBAA)
    pub fn parse_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
                let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
                let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
                Some(Self { r, g, b, a: 255 })
            }
            4 => {
                let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
                let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
                let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
                let a = u8::from_str_radix(&hex[3..4], 16).ok()? * 17;
                Some(Self { r, g, b, a })
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Self { r, g, b, a: 255 })
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(Self { r, g, b, a })
            }
            _ => None,
        }
    }
    
    /// To BGRA format for framebuffer
    pub fn to_bgra(&self) -> u32 {
        ((self.a as u32) << 24) | ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }
}

impl Default for Color {
    fn default() -> Self {
        Color::TRANSPARENT
    }
}

/// CSS border style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorderStyle {
    None,
    Solid,
    Dashed,
    Dotted,
    Double,
}

impl Default for BorderStyle {
    fn default() -> Self { BorderStyle::None }
}

/// CSS overflow property
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overflow {
    Visible,
    Hidden,
    Scroll,
    Auto,
}

impl Default for Overflow {
    fn default() -> Self { Overflow::Visible }
}

/// Complete set of CSS properties for an element
#[derive(Debug, Clone, Default)]
pub struct ComputedStyle {
    // Display & positioning
    pub display: Display,
    pub position: Position,
    pub top: Length,
    pub right: Length,
    pub bottom: Length,
    pub left: Length,
    pub z_index: Option<i32>,
    
    // Box model
    pub width: Length,
    pub height: Length,
    pub min_width: Length,
    pub min_height: Length,
    pub max_width: Length,
    pub max_height: Length,
    
    pub margin_top: Length,
    pub margin_right: Length,
    pub margin_bottom: Length,
    pub margin_left: Length,
    
    pub padding_top: Length,
    pub padding_right: Length,
    pub padding_bottom: Length,
    pub padding_left: Length,
    
    pub border_top_width: f32,
    pub border_right_width: f32,
    pub border_bottom_width: f32,
    pub border_left_width: f32,
    
    pub border_top_style: BorderStyle,
    pub border_right_style: BorderStyle,
    pub border_bottom_style: BorderStyle,
    pub border_left_style: BorderStyle,
    
    pub border_top_color: Color,
    pub border_right_color: Color,
    pub border_bottom_color: Color,
    pub border_left_color: Color,
    
    pub box_sizing: BoxSizing,
    
    // Flexbox
    pub flex_direction: FlexDirection,
    pub flex_wrap: FlexWrap,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    pub align_self: AlignSelf,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_basis: Length,
    pub order: i32,
    pub gap: Length,
    
    // Visual
    pub background_color: Color,
    pub color: Color,
    pub opacity: f32,
    
    // Text
    pub font_size: Length,
    pub font_weight: u16,
    pub line_height: Length,
    pub text_align: TextAlign,
    
    // Overflow
    pub overflow_x: Overflow,
    pub overflow_y: Overflow,
}

/// Text alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Right,
    Center,
    Justify,
}

impl Default for TextAlign {
    fn default() -> Self { TextAlign::Left }
}

/// Style value for parsing
#[derive(Debug, Clone, PartialEq)]
pub enum StyleValue {
    Display(Display),
    Position(Position),
    Length(Length),
    Color(Color),
    Number(f32),
    Integer(i32),
    Text(String),
    None,
}

/// Parse a CSS length value
pub fn parse_length(value: &str) -> Length {
    let value = value.trim();
    
    if value == "auto" {
        return Length::Auto;
    }
    if value == "0" {
        return Length::Zero;
    }
    
    // Try to parse numeric value with unit
    if value.ends_with("px") {
        if let Ok(v) = value[..value.len()-2].parse::<f32>() {
            return Length::Px(v);
        }
    } else if value.ends_with("em") {
        if let Ok(v) = value[..value.len()-2].parse::<f32>() {
            return Length::Em(v);
        }
    } else if value.ends_with("rem") {
        if let Ok(v) = value[..value.len()-3].parse::<f32>() {
            return Length::Rem(v);
        }
    } else if value.ends_with('%') {
        if let Ok(v) = value[..value.len()-1].parse::<f32>() {
            return Length::Percent(v);
        }
    } else if let Ok(v) = value.parse::<f32>() {
        // Unitless numbers treated as px
        return Length::Px(v);
    }
    
    Length::Auto
}

/// Parse display property
pub fn parse_display(value: &str) -> Display {
    match value.trim() {
        "block" => Display::Block,
        "inline" => Display::Inline,
        "inline-block" => Display::InlineBlock,
        "flex" => Display::Flex,
        "grid" => Display::Grid,
        "none" => Display::None,
        "table" => Display::Table,
        "table-row" => Display::TableRow,
        "table-cell" => Display::TableCell,
        _ => Display::Inline,
    }
}

/// Parse position property
pub fn parse_position(value: &str) -> Position {
    match value.trim() {
        "static" => Position::Static,
        "relative" => Position::Relative,
        "absolute" => Position::Absolute,
        "fixed" => Position::Fixed,
        "sticky" => Position::Sticky,
        _ => Position::Static,
    }
}

/// Parse flex direction
pub fn parse_flex_direction(value: &str) -> FlexDirection {
    match value.trim() {
        "row" => FlexDirection::Row,
        "row-reverse" => FlexDirection::RowReverse,
        "column" => FlexDirection::Column,
        "column-reverse" => FlexDirection::ColumnReverse,
        _ => FlexDirection::Row,
    }
}

/// Parse justify-content
pub fn parse_justify_content(value: &str) -> JustifyContent {
    match value.trim() {
        "flex-start" => JustifyContent::FlexStart,
        "flex-end" => JustifyContent::FlexEnd,
        "center" => JustifyContent::Center,
        "space-between" => JustifyContent::SpaceBetween,
        "space-around" => JustifyContent::SpaceAround,
        "space-evenly" => JustifyContent::SpaceEvenly,
        _ => JustifyContent::FlexStart,
    }
}

/// Parse align-items
pub fn parse_align_items(value: &str) -> AlignItems {
    match value.trim() {
        "flex-start" => AlignItems::FlexStart,
        "flex-end" => AlignItems::FlexEnd,
        "center" => AlignItems::Center,
        "stretch" => AlignItems::Stretch,
        "baseline" => AlignItems::Baseline,
        _ => AlignItems::Stretch,
    }
}

/// Parse box-sizing
pub fn parse_box_sizing(value: &str) -> BoxSizing {
    match value.trim() {
        "content-box" => BoxSizing::ContentBox,
        "border-box" => BoxSizing::BorderBox,
        _ => BoxSizing::ContentBox,
    }
}

/// Parse border style
pub fn parse_border_style(value: &str) -> BorderStyle {
    match value.trim() {
        "none" => BorderStyle::None,
        "solid" => BorderStyle::Solid,
        "dashed" => BorderStyle::Dashed,
        "dotted" => BorderStyle::Dotted,
        "double" => BorderStyle::Double,
        _ => BorderStyle::None,
    }
}

/// Parse text align
pub fn parse_text_align(value: &str) -> TextAlign {
    match value.trim() {
        "left" => TextAlign::Left,
        "right" => TextAlign::Right,
        "center" => TextAlign::Center,
        "justify" => TextAlign::Justify,
        _ => TextAlign::Left,
    }
}