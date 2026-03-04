//! Content extraction (Readability-style)

use alloc::string::String;
use alloc::vec::Vec;
use crate::html::{extract_title, extract_body_text, strip_tags};
use crate::http::HttpResponse;

/// Extracted content
#[derive(Debug, Default, Clone)]
pub struct ExtractedContent {
    pub title: String,
    pub content: String,
    pub author: Option<String>,
    pub published_date: Option<String>,
    pub url: String,
}

/// Content extraction error
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExtractionError {
    HttpError,
    ParseError,
    EmptyContent,
}

/// Content extractor with readability-style heuristics
pub struct ContentExtractor;

impl ContentExtractor {
    /// Extract readable content from HTTP response
    pub fn from_response(response: &HttpResponse, url: &str) -> Result<ExtractedContent, ExtractionError> {
        if !response.is_success() {
            return Err(ExtractionError::HttpError);
        }
        
        let body_text = response.body_text()
            .ok_or(ExtractionError::ParseError)?;
        
        // If HTML, extract text content
        let content = if response.is_html() {
            extract_body_text(&body_text)
        } else if response.is_text() {
            body_text
        } else {
            // Binary or unknown - try to extract as text anyway
            body_text
        };
        
        if content.trim().is_empty() {
            return Err(ExtractionError::EmptyContent);
        }
        
        let title = extract_title(&body_text)
            .unwrap_or_else(|| "Untitled".to_string());
        
        Ok(ExtractedContent {
            title,
            content,
            author: None,
            published_date: None,
            url: url.to_string(),
        })
    }
    
    /// Extract from raw HTTP response bytes
    pub fn from_bytes(data: &[u8], url: &str) -> Result<ExtractedContent, ExtractionError> {
        let response = HttpResponse::parse(data)
            .map_err(|_| ExtractionError::ParseError)?;
        Self::from_response(&response, url)
    }
    
    /// Quick extract - just get text content
    pub fn extract_text(html: &str) -> String {
        extract_body_text(html)
    }
    
    /// Get first paragraph as summary
    pub fn extract_summary(content: &str, max_len: usize) -> String {
        let text = strip_tags(content);
        if text.len() <= max_len {
            text
        } else {
            let mut end = max_len;
            while end > 0 && !text.is_char_boundary(end) {
                end -= 1;
            }
            format!("{}...", &text[..end])
        }
    }
}

/// Text document for display
#[derive(Debug, Clone)]
pub struct TextDocument {
    pub title: String,
    pub lines: Vec<String>,
    pub url: String,
}

impl TextDocument {
    /// Create from extracted content, wrapping lines
    pub fn from_content(content: &ExtractedContent, wrap_width: usize) -> Self {
        let lines = wrap_text(&content.content, wrap_width);
        
        Self {
            title: content.title.clone(),
            lines,
            url: content.url.clone(),
        }
    }
    
    /// Get total line count
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }
    
    /// Get lines for display range
    pub fn get_lines(&self, start: usize, count: usize) -> &[String] {
        let end = (start + count).min(self.lines.len());
        &self.lines[start..end]
    }
}

/// Simple word-wrapping for text
fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::with_capacity(width);
    
    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line.push_str(word);
        } else if current_line.len() + 1 + word.len() <= width {
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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_wrap_text() {
        let text = "The quick brown fox jumps over the lazy dog";
        let lines = wrap_text(text, 20);
        
        assert!(!lines.is_empty());
        assert!(lines.iter().all(|l| l.len() <= 20));
    }
    
    #[test]
    fn test_extract_summary() {
        let text = "This is a very long text that needs to be summarized for display purposes";
        let summary = ContentExtractor::extract_summary(text, 20);
        
        assert!(summary.len() <= 23); // 20 + "..."
        assert!(summary.ends_with("..."));
    }
}