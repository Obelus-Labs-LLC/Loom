//! HTML processing - tag stripping and text extraction

/// Strip HTML tags, preserve text content
pub fn strip_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len() / 2);
    let mut in_tag = false;
    let mut in_script = false;
    let mut in_style = false;
    let mut chars = html.chars().peekable();
    
    while let Some(ch) = chars.next() {
        // Check for script/style tag start
        if ch == '<' {
            let remaining: String = chars.by_ref().take(10).collect();
            let tag_start = remaining.to_lowercase();
            
            if tag_start.starts_with("script") {
                in_script = true;
            } else if tag_start.starts_with("style") {
                in_style = true;
            } else if tag_start.starts_with("/script") {
                in_script = false;
                continue;
            } else if tag_start.starts_with("/style") {
                in_style = false;
                continue;
            }
            
            // Put chars back (simplified - just skip tag)
            in_tag = true;
            continue;
        }
        
        if in_script || in_style {
            continue;
        }
        
        if ch == '>' && in_tag {
            in_tag = false;
            continue;
        }
        
        if !in_tag {
            // Convert common HTML entities
            if ch == '&' {
                match chars.peek() {
                    Some('l') => { // &lt;
                        chars.next();
                        if chars.next() == Some(';') {
                            result.push('<');
                        }
                    }
                    Some('g') => { // &gt;
                        chars.next();
                        if chars.next() == Some(';') {
                            result.push('>');
                        }
                    }
                    Some('a') => { // &amp;
                        chars.next();
                        if chars.next() == Some('m') && chars.next() == Some('p') && chars.next() == Some(';') {
                            result.push('&');
                        }
                    }
                    Some('n') => { // &nbsp;
                        chars.next();
                        if chars.next() == Some('b') && chars.next() == Some('s') && chars.next() == Some('p') && chars.next() == Some(';') {
                            result.push(' ');
                        }
                    }
                    _ => result.push(ch),
                }
            } else if ch.is_whitespace() {
                // Collapse whitespace
                if !result.ends_with(' ') && !result.is_empty() {
                    result.push(' ');
                }
            } else {
                result.push(ch);
            }
        }
    }
    
    // Trim leading/trailing whitespace
    result.trim().to_string()
}

/// Extract text content from specific HTML elements
pub fn extract_text_by_tag(html: &str, tag: &str) -> Vec<String> {
    let mut results = Vec::new();
    let open_tag = format!("<{}", tag);
    let close_tag = format!("</{}>", tag);
    
    let mut pos = 0;
    while let Some(start) = html[pos..].to_lowercase().find(&open_tag.to_lowercase()) {
        let absolute_start = pos + start;
        
        // Find end of opening tag
        if let Some(tag_end) = html[absolute_start..].find('>') {
            let content_start = absolute_start + tag_end + 1;
            
            // Find closing tag
            if let Some(content_end) = html[content_start..].to_lowercase().find(&close_tag.to_lowercase()) {
                let content = &html[content_start..content_start + content_end];
                results.push(strip_tags(content));
                pos = content_start + content_end + close_tag.len();
            } else {
                break;
            }
        } else {
            break;
        }
    }
    
    results
}

/// Extract title from HTML
pub fn extract_title(html: &str) -> Option<String> {
    extract_text_by_tag(html, "title").into_iter().next()
}

/// Extract body text from HTML (simplified - just strips all tags)
pub fn extract_body_text(html: &str) -> String {
    // Try to find <body> tag content
    if let Some(body_start) = html.to_lowercase().find("<body") {
        if let Some(body_tag_end) = html[body_start..].find('>') {
            let content_start = body_start + body_tag_end + 1;
            if let Some(body_end) = html[content_start..].to_lowercase().find("</body>") {
                let body_content = &html[content_start..content_start + body_end];
                return strip_tags(body_content);
            }
        }
    }
    
    // Fallback: strip all tags from entire document
    strip_tags(html)
}

/// Simple HTML entity decoder
pub fn decode_entities(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars();
    
    while let Some(ch) = chars.next() {
        if ch == '&' {
            let mut entity = String::new();
            for c in chars.by_ref() {
                if c == ';' {
                    break;
                }
                entity.push(c);
            }
            
            match entity.as_str() {
                "lt" => result.push('<'),
                "gt" => result.push('>'),
                "amp" => result.push('&'),
                "quot" => result.push('"'),
                "apos" => result.push('\''),
                "nbsp" => result.push(' '),
                "ndash" => result.push('–'),
                "mdash" => result.push('—'),
                _ => {
                    // Numeric entities
                    if entity.starts_with('#') {
                        if let Some(code) = entity[1..].parse::<u32>().ok() {
                            if let Some(decoded) = char::from_u32(code) {
                                result.push(decoded);
                            }
                        }
                    } else {
                        result.push('&');
                        result.push_str(&entity);
                        result.push(';');
                    }
                }
            }
        } else {
            result.push(ch);
        }
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_strip_tags() {
        let html = "<p>Hello <b>world</b>!</p>";
        assert_eq!(strip_tags(html), "Hello world!");
    }
    
    #[test]
    fn test_extract_title() {
        let html = "<html><head><title>Test Page</title></head><body>Content</body></html>";
        assert_eq!(extract_title(html), Some("Test Page".to_string()));
    }
    
    #[test]
    fn test_strip_script_and_style() {
        let html = r#"<p>Text</p><script>alert('hi');</script><style>.class{color:red}</style><p>More</p>"#;
        assert_eq!(strip_tags(html), "Text More");
    }
}