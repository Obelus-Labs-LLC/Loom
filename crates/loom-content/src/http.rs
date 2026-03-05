//! HTTP response parsing

/// Parsed HTTP response
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub version: String,
    pub status_code: u16,
    pub status_text: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

/// HTTP parse error
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HttpParseError {
    InvalidFormat,
    InvalidStatusLine,
    InvalidHeader,
    InvalidEncoding,
    Incomplete,
}

impl HttpResponse {
    /// Parse raw HTTP response bytes
    pub fn parse(data: &[u8]) -> Result<Self, HttpParseError> {
        // Find end of headers (double CRLF)
        let header_end = data.windows(4)
            .position(|w| w == b"\r\n\r\n")
            .or_else(|| data.windows(2).position(|w| w == b"\n\n"))
            .ok_or(HttpParseError::Incomplete)?;
        
        let header_bytes = &data[..header_end];
        let body_start = if data[header_end..].starts_with(b"\r\n\r\n") {
            header_end + 4
        } else {
            header_end + 2
        };
        let body = data[body_start..].to_vec();
        
        // Parse headers as string
        let header_text = str::from_utf8(header_bytes)
            .map_err(|_| HttpParseError::InvalidEncoding)?;
        
        let mut lines = header_text.lines();
        
        // Parse status line
        let status_line = lines.next().ok_or(HttpParseError::InvalidStatusLine)?;
        let (version, status_code, status_text) = Self::parse_status_line(status_line)?;
        
        // Parse headers
        let mut headers = Vec::new();
        for line in lines {
            let line: &str = line;
            if line.is_empty() {
                continue;
            }
            if let Some((name, value)) = Self::parse_header(line) {
                headers.push((name, value));
            }
        }
        
        Ok(HttpResponse {
            version,
            status_code,
            status_text,
            headers,
            body,
        })
    }
    
    /// Parse status line: "HTTP/1.1 200 OK"
    fn parse_status_line(line: &str) -> Result<(String, u16, String), HttpParseError> {
        let parts: Vec<&str> = line.splitn(3, ' ').collect();
        if parts.len() < 2 {
            return Err(HttpParseError::InvalidStatusLine);
        }
        
        let version = parts[0].to_string();
        let status_code = parts[1].parse::<u16>()
            .map_err(|_| HttpParseError::InvalidStatusLine)?;
        let status_text = parts.get(2).unwrap_or(&"").to_string();
        
        Ok((version, status_code, status_text))
    }
    
    /// Parse header line: "Content-Type: text/html"
    fn parse_header(line: &str) -> Option<(String, String)> {
        let pos = line.find(':')?;
        let name = line[..pos].trim().to_lowercase();
        let value = line[pos+1..].trim().to_string();
        Some((name, value))
    }
    
    /// Get header value by name (case-insensitive)
    pub fn header(&self, name: &str) -> Option<&str> {
        let name_lower = name.to_lowercase();
        self.headers.iter()
            .find(|(k, _): &&(String, String)| k == &name_lower)
            .map(|(_, v): &(String, String)| v.as_str())
    }
    
    /// Get body as text (UTF-8)
    pub fn body_text(&self) -> Option<String> {
        String::from_utf8(self.body.clone()).ok()
    }
    
    /// Check if response is successful (2xx)
    pub fn is_success(&self) -> bool {
        self.status_code >= 200 && self.status_code < 300
    }
    
    /// Check if response is redirect (3xx)
    pub fn is_redirect(&self) -> bool {
        self.status_code >= 300 && self.status_code < 400
    }
    
    /// Get redirect location if present
    pub fn redirect_location(&self) -> Option<&str> {
        self.header("location")
    }
    
    /// Get content type header
    pub fn content_type(&self) -> Option<&str> {
        self.header("content-type")
    }
    
    /// Check if content is HTML
    pub fn is_html(&self) -> bool {
        self.content_type()
            .map(|ct: &str| ct.contains("text/html"))
            .unwrap_or(false)
    }
    
    /// Check if content is plain text
    pub fn is_text(&self) -> bool {
        self.content_type()
            .map(|ct: &str| ct.starts_with("text/"))
            .unwrap_or(false)
    }
}

/// Quick parse - extract just the body from HTTP response
pub fn extract_body(data: &[u8]) -> Result<Vec<u8>, HttpParseError> {
    let response = HttpResponse::parse(data)?;
    Ok(response.body)
}

/// Quick parse - extract body as text
pub fn extract_body_text(data: &[u8]) -> Result<String, HttpParseError> {
    let response = HttpResponse::parse(data)?;
    response.body_text().ok_or(HttpParseError::InvalidEncoding)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_simple_response() {
        let response = b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n<html><body>Hello</body></html>";
        let parsed = HttpResponse::parse(response).unwrap();
        
        assert_eq!(parsed.status_code, 200);
        assert_eq!(parsed.body_text(), Some("<html><body>Hello</body></html>".to_string()));
    }
    
    #[test]
    fn test_parse_headers() {
        let response = b"HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\nContent-Length: 9\r\n\r\nNot found";
        let parsed = HttpResponse::parse(response).unwrap();
        
        assert_eq!(parsed.status_code, 404);
        assert_eq!(parsed.header("content-type"), Some("text/plain"));
        assert_eq!(parsed.header("content-length"), Some("9"));
    }
}