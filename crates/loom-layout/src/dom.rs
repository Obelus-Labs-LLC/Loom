//! DOM parsing with html5ever

use html5ever::parse_document;
use html5ever::tendril::TendrilSink;
use markup5ever_rcdom::{Handle, RcDom};

/// Parsed DOM document
pub struct Document {
    pub root: Handle,
}

impl Document {
    pub fn parse(html: &str) -> anyhow::Result<Self> {
        let dom = parse_document(RcDom::default(), Default::default())
            .from_utf8()
            .read_from(&mut html.as_bytes())?;
        
        Ok(Self { root: dom.document })
    }
}
