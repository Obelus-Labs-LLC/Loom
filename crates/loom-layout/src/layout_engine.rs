//! Layout engine: block layout, flexbox, positioning

use alloc::vec::Vec;
use alloc::string::String;

use crate::css_types::*;
use crate::style_engine::*;
use crate::dom::ElementData;
use crate::css_types::ComputedStyle;

/// Layout box - positioned rectangle with style
#[derive(Debug, Clone)]
pub struct LayoutBox {
    // Position relative to containing block
    pub x: f32,
    pub y: f32,
    // Content box dimensions
    pub content_width: f32,
    pub content_height: f32,
    // Padding
    pub padding_top: f32,
    pub padding_right: f32,
    pub padding_bottom: f32,
    pub padding_left: f32,
    // Border
    pub border_top: f32,
    pub border_right: f32,
    pub border_bottom: f32,
    pub border_left: f32,
    // Margin
    pub margin_top: f32,
    pub margin_right: f32,
    pub margin_bottom: f32,
    pub margin_left: f32,
    // Computed style
    pub style: ComputedStyle,
    // Element info
    pub element: Option<ElementData>,
    // Text content (for inline boxes)
    pub text_content: Option<String>,
}

impl LayoutBox {
    pub fn new(style: ComputedStyle) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            content_width: 0.0,
            content_height: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            border_top: 0.0,
            border_right: 0.0,
            border_bottom: 0.0,
            border_left: 0.0,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
            style,
            element: None,
            text_content: None,
        }
    }
    
    /// Total width including padding and border
    pub fn padding_box_width(&self) -> f32 {
        self.content_width + self.padding_left + self.padding_right
    }
    
    pub fn padding_box_height(&self) -> f32 {
        self.content_height + self.padding_top + self.padding_bottom
    }
    
    /// Total width including border
    pub fn border_box_width(&self) -> f32 {
        self.padding_box_width() + self.border_left + self.border_right
    }
    
    pub fn border_box_height(&self) -> f32 {
        self.padding_box_height() + self.border_top + self.border_bottom
    }
    
    /// Total width including margin
    pub fn margin_box_width(&self) -> f32 {
        self.border_box_width() + self.margin_left + self.margin_right
    }
    
    pub fn margin_box_height(&self) -> f32 {
        self.border_box_height() + self.margin_top + self.margin_bottom
    }
    
    /// Set box dimensions based on style and available space
    pub fn resolve_dimensions(&mut self, available_width: f32, available_height: f32, font_size: f32) {
        // Resolve margins
        self.margin_top = self.style.margin_top.to_px(available_width, font_size);
        self.margin_right = self.style.margin_right.to_px(available_width, font_size);
        self.margin_bottom = self.style.margin_bottom.to_px(available_width, font_size);
        self.margin_left = self.style.margin_left.to_px(available_width, font_size);
        
        // Resolve padding
        self.padding_top = self.style.padding_top.to_px(available_width, font_size);
        self.padding_right = self.style.padding_right.to_px(available_width, font_size);
        self.padding_bottom = self.style.padding_bottom.to_px(available_width, font_size);
        self.padding_left = self.style.padding_left.to_px(available_width, font_size);
        
        // Resolve borders
        self.border_top = self.style.border_top_width;
        self.border_right = self.style.border_right_width;
        self.border_bottom = self.style.border_bottom_width;
        self.border_left = self.style.border_left_width;
        
        // Resolve width
        if self.style.width.is_auto() {
            // Auto width fills available space minus margins
            self.content_width = available_width - self.margin_left - self.margin_right
                - self.padding_left - self.padding_right
                - self.border_left - self.border_right;
            self.content_width = self.content_width.max(0.0);
        } else {
            self.content_width = self.style.width.to_px(available_width, font_size);
            
            // Handle box-sizing
            if self.style.box_sizing == BoxSizing::BorderBox {
                // Width includes padding and border
                self.content_width -= self.padding_left + self.padding_right
                    + self.border_left + self.border_right;
                self.content_width = self.content_width.max(0.0);
            }
        }
        
        // Apply min/max width constraints
        let min_width = self.style.min_width.to_px(available_width, font_size);
        let max_width = if self.style.max_width.is_auto() {
            f32::INFINITY
        } else {
            self.style.max_width.to_px(available_width, font_size)
        };
        self.content_width = self.content_width.clamp(min_width, max_width);
        
        // Height - auto means content-dependent (set later)
        if !self.style.height.is_auto() {
            self.content_height = self.style.height.to_px(available_height, font_size);
            
            if self.style.box_sizing == BoxSizing::BorderBox {
                self.content_height -= self.padding_top + self.padding_bottom
                    + self.border_top + self.border_bottom;
                self.content_height = self.content_height.max(0.0);
            }
        }
    }
}

/// Layout tree node
#[derive(Debug)]
pub struct LayoutNode {
    pub box_: LayoutBox,
    pub children: Vec<LayoutNode>,
}

impl LayoutNode {
    pub fn new(style: ComputedStyle) -> Self {
        Self {
            box_: LayoutBox::new(style),
            children: Vec::new(),
        }
    }
    
    pub fn add_child(&mut self, child: LayoutNode) {
        self.children.push(child);
    }
}

/// Layout context
pub struct LayoutContext {
    pub viewport_width: f32,
    pub viewport_height: f32,
    pub root_font_size: f32,
}

impl LayoutContext {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            viewport_width: width,
            viewport_height: height,
            root_font_size: 16.0,
        }
    }
}

/// Main layout algorithm
pub fn layout_tree(root: &mut LayoutNode, ctx: &LayoutContext) {
    // Layout root
    layout_node(root, ctx.viewport_width, ctx.viewport_height, ctx.root_font_size, &Position::Static, 0.0, 0.0, ctx);
}

/// Layout a single node
fn layout_node(node: &mut LayoutNode, available_width: f32, available_height: f32, 
               font_size: f32, containing_position: &Position, containing_x: f32, containing_y: f32,
               viewport: &LayoutContext) {
    let style = node.box_.style.clone();
    
    // Resolve dimensions
    node.box_.resolve_dimensions(available_width, available_height, font_size);
    
    // Handle positioning
    match style.position {
        Position::Static | Position::Relative => {
            // Normal flow
            match style.display {
                Display::Block | Display::Flex => {
                    layout_block(node, available_width, font_size, containing_position, containing_x, containing_y, viewport);
                }
                Display::Inline | Display::InlineBlock => {
                    layout_inline(node, available_width, font_size, viewport);
                }
                Display::None => {
                    // No layout, element is not displayed
                    node.box_.content_width = 0.0;
                    node.box_.content_height = 0.0;
                }
                _ => {
                    layout_block(node, available_width, font_size, containing_position, containing_x, containing_y, viewport);
                }
            }
        }
        Position::Absolute => {
            // Absolute positioning
            layout_absolute(node, available_width, available_height, font_size, containing_x, containing_y, viewport);
        }
        Position::Fixed => {
            // Fixed positioning (relative to viewport)
            layout_fixed(node, viewport, font_size);
        }
        _ => {
            layout_block(node, available_width, font_size, containing_position, containing_x, containing_y, viewport);
        }
    }
}

/// Block layout algorithm
fn layout_block(node: &mut LayoutNode, available_width: f32, font_size: f32,
                _containing_position: &Position, containing_x: f32, containing_y: f32,
                viewport: &LayoutContext) {
    let style = node.box_.style.clone();
    
    // Content box width available for children
    let child_available_width = node.box_.content_width;
    
    // Layout children
    if style.display == Display::Flex {
        layout_flexbox(node, child_available_width, font_size, viewport);
    } else {
        // Normal block flow
        let mut current_y = 0.0f32;
        
        for child in &mut node.children {
            layout_node(child, child_available_width, f32::INFINITY, font_size, &Position::Static, containing_x, containing_y, viewport);
            
            // Position child
            child.box_.x = 0.0;
            child.box_.y = current_y;
            
            // Advance y by child's margin box height
            current_y += child.box_.margin_box_height();
        }
        
        // Calculate content height if auto
        if node.box_.style.height.is_auto() {
            node.box_.content_height = current_y;
            
            // Apply min/max height
            let min_height = node.box_.style.min_height.to_px(available_width, font_size);
            let max_height = if node.box_.style.max_height.is_auto() {
                f32::INFINITY
            } else {
                node.box_.style.max_height.to_px(available_width, font_size)
            };
            node.box_.content_height = node.box_.content_height.clamp(min_height, max_height);
        }
    }
    
    // Handle relative positioning offset
    if style.position == Position::Relative {
        node.box_.x += style.left.to_px(available_width, font_size);
        node.box_.y += style.top.to_px(available_width, font_size);
    }
}

/// Inline layout (simplified - treats as block for now)
fn layout_inline(node: &mut LayoutNode, available_width: f32, font_size: f32, viewport: &LayoutContext) {
    // For now, inline elements are treated similarly to blocks
    // In a full implementation, this would do text wrapping and inline box handling
    
    let mut current_y = 0.0f32;
    
    for child in &mut node.children {
        layout_node(child, available_width, f32::INFINITY, font_size, &Position::Static, 0.0, 0.0, viewport);
        child.box_.x = 0.0;
        child.box_.y = current_y;
        current_y += child.box_.margin_box_height();
    }
    
    if node.box_.style.height.is_auto() {
        node.box_.content_height = current_y;
    }
}

/// Flexbox layout
fn layout_flexbox(node: &mut LayoutNode, available_width: f32, font_size: f32, viewport: &LayoutContext) {
    let style = &node.box_.style;
    let is_row = matches!(style.flex_direction, FlexDirection::Row | FlexDirection::RowReverse);
    let is_wrap = style.flex_wrap == FlexWrap::Wrap;
    let justify = style.justify_content;
    let align_items = style.align_items;
    let gap = style.gap.to_px(available_width, font_size);
    
    // Main axis size
    let main_size = if is_row { available_width } else { node.box_.content_height };
    let cross_size = if is_row { node.box_.content_height } else { available_width };
    
    // Layout all children to determine their flex basis
    let mut flex_items: Vec<(usize, &mut LayoutNode, f32, f32)> = Vec::new();
    let mut total_main_size = 0.0f32;
    let mut max_cross_size = 0.0f32;
    
    for (i, child) in node.children.iter_mut().enumerate() {
        // Determine flex basis
        let basis = if child.box_.style.flex_basis.is_auto() {
            // Use width/height as basis
            if is_row {
                child.box_.style.width.to_px(available_width, font_size)
            } else {
                child.box_.style.height.to_px(cross_size, font_size)
            }
        } else {
            child.box_.style.flex_basis.to_px(main_size, font_size)
        };
        
        // Layout child with basis as available size
        let child_available = if is_row { basis } else { available_width };
        layout_node(child, child_available, f32::INFINITY, font_size, &Position::Static, 0.0, 0.0, viewport);
        
        let main = if is_row {
            child.box_.margin_box_width()
        } else {
            child.box_.margin_box_height()
        };
        
        let cross = if is_row {
            child.box_.margin_box_height()
        } else {
            child.box_.margin_box_width()
        };
        
        flex_items.push((i, child, main, cross));
        total_main_size += main + gap;
        max_cross_size = max_cross_size.max(cross);
    }
    
    // Remove trailing gap
    if !flex_items.is_empty() {
        total_main_size -= gap;
    }
    
    // Calculate remaining space for flex grow/shrink
    let remaining = main_size - total_main_size;
    
    // Distribute remaining space
    if remaining > 0.0 {
        // Flex grow
        let total_grow: f32 = flex_items.iter()
            .map(|(_, node, _, _)| node.box_.style.flex_grow)
            .sum();
        
        if total_grow > 0.0 {
            for (_, node, main, _) in &mut flex_items {
                let grow = node.box_.style.flex_grow;
                let extra = (grow / total_grow) * remaining;
                // Adjust main size
                if is_row {
                    node.box_.content_width += extra;
                } else {
                    node.box_.content_height += extra;
                }
            }
        }
    } else if remaining < 0.0 {
        // Flex shrink
        let total_shrink: f32 = flex_items.iter()
            .map(|(_, node, _, _)| node.box_.style.flex_shrink)
            .sum();
        
        if total_shrink > 0.0 {
            for (_, node, main, _) in &mut flex_items {
                let shrink = node.box_.style.flex_shrink;
                let reduction = (shrink / total_shrink) * (-remaining);
                // Reduce main size
                if is_row {
                    node.box_.content_width = (node.box_.content_width - reduction).max(0.0);
                } else {
                    node.box_.content_height = (node.box_.content_height - reduction).max(0.0);
                }
            }
        }
    }
    
    // Position items along main axis
    let mut main_pos = match justify {
        JustifyContent::FlexStart => 0.0,
        JustifyContent::Center => remaining / 2.0,
        JustifyContent::FlexEnd => remaining,
        JustifyContent::SpaceBetween => {
            if flex_items.len() > 1 {
                0.0
            } else {
                remaining / 2.0
            }
        }
        JustifyContent::SpaceAround => {
            let space = if flex_items.is_empty() {
                0.0
            } else {
                remaining / flex_items.len() as f32
            };
            space / 2.0
        }
        JustifyContent::SpaceEvenly => {
            let n = (flex_items.len() + 1) as f32;
            remaining / n
        }
    };
    
    let space_between = match justify {
        JustifyContent::SpaceBetween if flex_items.len() > 1 => {
            remaining / (flex_items.len() - 1) as f32
        }
        JustifyContent::SpaceAround if !flex_items.is_empty() => {
            remaining / flex_items.len() as f32
        }
        JustifyContent::SpaceEvenly if !flex_items.is_empty() => {
            remaining / (flex_items.len() + 1) as f32
        }
        _ => 0.0,
    };
    
    for (_, child, main, cross) in &mut flex_items {
        // Calculate cross-axis position (alignment)
        let cross_pos = match child.box_.style.align_self {
            AlignSelf::Auto => match align_items {
                AlignItems::FlexStart => 0.0,
                AlignItems::FlexEnd => max_cross_size - *cross,
                AlignItems::Center => (max_cross_size - *cross) / 2.0,
                AlignItems::Stretch => 0.0,
                _ => 0.0,
            }
            AlignSelf::FlexStart => 0.0,
            AlignSelf::FlexEnd => max_cross_size - *cross,
            AlignSelf::Center => (max_cross_size - *cross) / 2.0,
            AlignSelf::Stretch => 0.0,
            _ => 0.0,
        };
        
        // Set position
        if is_row {
            child.box_.x = main_pos + child.box_.margin_left;
            child.box_.y = cross_pos + child.box_.margin_top;
        } else {
            child.box_.x = cross_pos + child.box_.margin_left;
            child.box_.y = main_pos + child.box_.margin_top;
        }
        
        main_pos += *main + gap + space_between;
    }
    
    // Update container height if auto
    if node.box_.style.height.is_auto() && is_row {
        node.box_.content_height = max_cross_size;
    }
}

/// Absolute positioning
fn layout_absolute(node: &mut LayoutNode, available_width: f32, available_height: f32, 
                   font_size: f32, containing_x: f32, containing_y: f32, viewport: &LayoutContext) {
    let style = &node.box_.style;
    
    // Position based on top/left/right/bottom
    if !style.left.is_auto() {
        node.box_.x = style.left.to_px(available_width, font_size);
    } else if !style.right.is_auto() {
        // Position from right
        let width = if style.width.is_auto() {
            100.0 // Default width
        } else {
            style.width.to_px(available_width, font_size)
        };
        node.box_.x = available_width - width - style.right.to_px(available_width, font_size);
    }
    
    if !style.top.is_auto() {
        node.box_.y = style.top.to_px(available_height, font_size);
    } else if !style.bottom.is_auto() {
        let height = if style.height.is_auto() {
            100.0
        } else {
            style.height.to_px(available_height, font_size)
        };
        node.box_.y = available_height - height - style.bottom.to_px(available_height, font_size);
    }
    
    // Layout children
    for child in &mut node.children {
        layout_node(child, node.box_.content_width, node.box_.content_height, 
                    font_size, &Position::Absolute, containing_x + node.box_.x, containing_y + node.box_.y, viewport);
    }
}

/// Fixed positioning (relative to viewport)
fn layout_fixed(node: &mut LayoutNode, ctx: &LayoutContext, font_size: f32) {
    let style = &node.box_.style;
    
    if !style.left.is_auto() {
        node.box_.x = style.left.to_px(ctx.viewport_width, font_size);
    } else if !style.right.is_auto() {
        node.box_.x = ctx.viewport_width - node.box_.content_width - style.right.to_px(ctx.viewport_width, font_size);
    }
    
    if !style.top.is_auto() {
        node.box_.y = style.top.to_px(ctx.viewport_height, font_size);
    } else if !style.bottom.is_auto() {
        node.box_.y = ctx.viewport_height - node.box_.content_height - style.bottom.to_px(ctx.viewport_height, font_size);
    }
    
    // Layout children
    for child in &mut node.children {
        layout_node(child, node.box_.content_width, node.box_.content_height, 
                    font_size, &Position::Fixed, node.box_.x, node.box_.y, ctx);
    }
}

/// Margin collapsing for block layout
pub fn collapse_margins(top: f32, bottom: f32) -> f32 {
    // Margins collapse to the larger of the two
    top.max(bottom)
}

/// Build layout tree from DOM
pub fn build_layout_tree(element: &ElementData, styles: &ComputedStyle) -> LayoutNode {
    let mut node = LayoutNode::new(styles.clone());
    node.box_.element = Some(element.clone());
    node
}

/// Simple text layout - wrap text into lines
pub fn layout_text(text: &str, max_width: f32, font_size: f32, char_width: f32) -> Vec<String> {
    let chars_per_line = (max_width / (font_size * char_width)) as usize;
    let mut lines = Vec::new();
    let mut current_line = String::new();
    
    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line.push_str(word);
        } else if current_line.len() + 1 + word.len() <= chars_per_line {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }
    
    if !current_line.is_empty() {
        lines.push(current_line);
    }
    
    lines
}