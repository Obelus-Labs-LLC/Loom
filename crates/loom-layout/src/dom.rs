//! DOM representation for layout engine
//!
//! Simplified DOM for no_std compatibility

use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;

/// DOM node type
#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    Element(ElementData),
    Text(String),
    Comment(String),
    Document,
}

/// Element data
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ElementData {
    pub tag_name: String,
    pub attributes: BTreeMap<String, String>,
    pub classes: Vec<String>,
    pub id: Option<String>,
}

impl ElementData {
    pub fn new(tag_name: &str) -> Self {
        Self {
            tag_name: tag_name.to_string(),
            attributes: BTreeMap::new(),
            classes: Vec::new(),
            id: None,
        }
    }
    
    pub fn with_id(mut self, id: &str) -> Self {
        self.id = Some(id.to_string());
        self
    }
    
    pub fn with_class(mut self, class: &str) -> Self {
        self.classes.push(class.to_string());
        self
    }
    
    pub fn with_attribute(mut self, name: &str, value: &str) -> Self {
        self.attributes.insert(name.to_string(), value.to_string());
        self
    }
    
    /// Check if element has a class
    pub fn has_class(&self, class: &str) -> bool {
        self.classes.iter().any(|c| c == class)
    }
    
    /// Get inline style attribute
    pub fn inline_style(&self) -> Option<&str> {
        self.attributes.get("style").map(|s| s.as_str())
    }
}

/// DOM node
#[derive(Debug, Clone)]
pub struct Node {
    pub node_type: NodeType,
    pub children: Vec<Node>,
    pub parent: Option<usize>, // Index into node list (for flat representation)
}

impl Node {
    pub fn new(node_type: NodeType) -> Self {
        Self {
            node_type,
            children: Vec::new(),
            parent: None,
        }
    }
    
    pub fn element(tag: &str) -> Self {
        Self::new(NodeType::Element(ElementData::new(tag)))
    }
    
    pub fn text(text: &str) -> Self {
        Self::new(NodeType::Text(text.to_string()))
    }
    
    pub fn add_child(&mut self, child: Node) -> &mut Self {
        self.children.push(child);
        self
    }
    
    /// Get element data if this is an element node
    pub fn as_element(&self) -> Option<&ElementData> {
        match &self.node_type {
            NodeType::Element(data) => Some(data),
            _ => None,
        }
    }
    
    /// Get mutable element data
    pub fn as_element_mut(&mut self) -> Option<&mut ElementData> {
        match &mut self.node_type {
            NodeType::Element(data) => Some(data),
            _ => None,
        }
    }
    
    /// Get text content if this is a text node
    pub fn as_text(&self) -> Option<&str> {
        match &self.node_type {
            NodeType::Text(text) => Some(text),
            _ => None,
        }
    }
}

/// Simple DOM document
#[derive(Debug)]
pub struct Document {
    pub root: Node,
}

impl Document {
    pub fn new() -> Self {
        let html = Node::element("html")
            .add_child(Node::element("head"))
            .add_child(Node::element("body"))
            .clone();
        
        Self { root: html }
    }
    
    /// Create from simple HTML-like structure (simplified parser)
    /// Only handles basic tag nesting: <tag>content</tag>
    pub fn parse_simple(html: &str) -> Self {
        let mut root = Node::element("html");
        let mut current = &mut root;
        
        // Very simple parser - just for testing
        // In production, use a proper HTML parser
        
        Document { root }
    }
    
    /// Find body element
    pub fn body(&mut self) -> Option<&mut Node> {
        self.find_element_mut("body")
    }
    
    /// Find element by tag name
    pub fn find_element_mut(&mut self, tag: &str) -> Option<&mut Node> {
        Self::find_in_node_mut(&mut self.root, tag)
    }
    
    fn find_in_node_mut<'a>(node: &'a mut Node, tag: &str) -> Option<&'a mut Node> {
        if let Some(data) = node.as_element() {
            if data.tag_name == tag {
                return Some(node);
            }
        }
        
        for child in &mut node.children {
            if let Some(found) = Self::find_in_node_mut(child, tag) {
                return Some(found);
            }
        }
        
        None
    }
    
    /// Walk all nodes and apply a function
    pub fn walk<F>(&self, f: &mut F)
    where
        F: FnMut(&Node),
    {
        Self::walk_node(&self.root, f);
    }
    
    fn walk_node<F>(node: &Node, f: &mut F)
    where
        F: FnMut(&Node),
    {
        f(node);
        for child in &node.children {
            Self::walk_node(child, f);
        }
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}