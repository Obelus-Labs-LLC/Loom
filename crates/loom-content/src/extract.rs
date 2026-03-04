//! Content extraction (Readability-style)

/// Extracted content
#[derive(Debug, Default)]
pub struct ExtractedContent {
    pub title: String,
    pub content: String,
    pub author: Option<String>,
    pub published_date: Option<String>,
}

/// Content extractor
pub struct ContentExtractor;

impl ContentExtractor {
    pub fn extract(html: &str) -> ExtractedContent {
        // TODO: Implement readability-style extraction
        ExtractedContent::default()
    }
}
