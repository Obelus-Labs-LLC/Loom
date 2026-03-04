//! Loom Browser - Phase L7: TLS + Interactive Browsing
//!
//! Build for FabricOS: cargo +nightly build -Z build-std=core,compiler_builtins,alloc

#![cfg_attr(not(feature = "std"), no_std, no_main)]

extern crate alloc;

#[cfg(not(feature = "std"))]
use core::alloc::{GlobalAlloc, Layout};
#[cfg(not(feature = "std"))]
use core::cell::UnsafeCell;

// Simple bump allocator for FabricOS
#[cfg(not(feature = "std"))]
pub struct BumpAllocator {
    heap: UnsafeCell<*mut u8>,
    end: UnsafeCell<*mut u8>,
}

#[cfg(not(feature = "std"))]
unsafe impl Sync for BumpAllocator {}

#[cfg(not(feature = "std"))]
impl BumpAllocator {
    pub const fn new() -> Self {
        Self {
            heap: UnsafeCell::new(0x800000 as *mut u8), // Start at 8MB
            end: UnsafeCell::new(0x1000000 as *mut u8), // End at 16MB
        }
    }
}

#[cfg(not(feature = "std"))]
unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let heap = *self.heap.get();
        let align = layout.align();
        let size = layout.size();
        let aligned = ((heap as usize + align - 1) & !(align - 1)) as *mut u8;
        let new_heap = aligned.add(size);
        if new_heap > *self.end.get() {
            return core::ptr::null_mut();
        }
        *self.heap.get() = new_heap;
        aligned
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[cfg(not(feature = "std"))]
#[global_allocator]
static ALLOCATOR: BumpAllocator = BumpAllocator::new();

mod os;

#[cfg(feature = "std")]
fn main() {
    println!("Loom Browser - Desktop mode");
    println!("Use 'cargo run' without --target for desktop");
}

// ============================================
// FABRICOS NO-STD IMPLEMENTATION - PHASE L7
// ============================================
#[cfg(not(feature = "std"))]
mod fabric_os {
    use super::*;
    use os::fabricsys::*;
    use alloc::format;
    use alloc::vec;
    use alloc::vec::Vec;
    use alloc::string::String;
    
    // Design System Colors (BGRA format)
    pub const C_BLACK: u32 = 0xFF000000;
    pub const C_DARK_GRAY: u32 = 0xFF1A1A1A;
    pub const C_MID_GRAY: u32 = 0xFF333333;
    pub const C_LIGHT_GRAY: u32 = 0xFF666666;
    pub const C_WHITE: u32 = 0xFFFFFFFF;
    pub const C_OFF_WHITE: u32 = 0xFFE8E6E3;
    pub const C_WARM_50: u32 = 0xFFF7F6F5;
    pub const C_WARM_100: u32 = 0xFFF0EDE9;
    pub const C_WARM_500: u32 = 0xFFD4A574;
    pub const C_WARM_600: u32 = 0xFFB8935F;
    pub const C_SUCCESS: u32 = 0xFF4CAF50;
    pub const C_WARNING: u32 = 0xFFFFA726;
    pub const C_ERROR: u32 = 0xFFE53935;
    pub const C_INFO: u32 = 0xFF42A5F5;
    
    // Display config
    pub const SCREEN_WIDTH: u32 = 1280;
    pub const SCREEN_HEIGHT: u32 = 800;
    pub const MARGIN_X: u32 = 20;
    pub const MARGIN_Y: u32 = 70; // Space for URL bar
    pub const URL_BAR_HEIGHT: u32 = 50;
    pub const STATUS_HEIGHT: u32 = 24;
    pub const LINE_HEIGHT: u32 = 16;
    pub const CHAR_WIDTH: u32 = 6;
    pub const CONTENT_WIDTH: u32 = SCREEN_WIDTH - 2 * MARGIN_X;
    pub const MAX_LINES: usize = ((SCREEN_HEIGHT - MARGIN_Y - STATUS_HEIGHT - 10) / LINE_HEIGHT) as usize;
    
    /// Browser modes
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum BrowserMode {
        Viewing,      // Normal page viewing with scroll
        UrlEditing,   // URL bar focused for input
        Loading,      // Page loading in progress
        Error,        // Error page displayed
    }
    
    /// Browser state
    pub struct Browser {
        pub url: String,
        pub page_title: String,
        pub content_lines: Vec<String>,
        pub scroll_offset: usize,
        pub mode: BrowserMode,
        pub url_buffer: String,
        pub cursor_pos: usize,
        pub status_message: String,
        pub history: Vec<String>,
        pub history_pos: usize,
    }
    
    impl Browser {
        pub fn new() -> Self {
            Self {
                url: String::from("http://example.com/"),
                page_title: String::new(),
                content_lines: Vec::new(),
                scroll_offset: 0,
                mode: BrowserMode::Viewing,
                url_buffer: String::from("http://example.com/"),
                cursor_pos: 19, // After "http://example.com/"
                status_message: String::from("Ready"),
                history: Vec::new(),
                history_pos: 0,
            }
        }
        
        pub fn navigate(&mut self, url: &str) {
            self.url = url.to_string();
            self.url_buffer = url.to_string();
            self.cursor_pos = url.len();
            self.mode = BrowserMode::Loading;
            self.status_message = format!("Loading {}...", url);
        }
        
        pub fn scroll_down(&mut self, lines: usize) {
            let max_scroll = self.content_lines.len().saturating_sub(MAX_LINES);
            self.scroll_offset = (self.scroll_offset + lines).min(max_scroll);
        }
        
        pub fn scroll_up(&mut self, lines: usize) {
            self.scroll_offset = self.scroll_offset.saturating_sub(lines);
        }
        
        pub fn page_down(&mut self) {
            self.scroll_down(MAX_LINES - 2);
        }
        
        pub fn page_up(&mut self) {
            self.scroll_up(MAX_LINES - 2);
        }
        
        pub fn set_content(&mut self, text: &str) {
            self.content_lines = wrap_text(text, (CONTENT_WIDTH / CHAR_WIDTH) as usize);
            self.scroll_offset = 0;
        }
        
        pub fn add_to_history(&mut self, url: &str) {
            if self.history.is_empty() || self.history.last().unwrap() != url {
                self.history.push(url.to_string());
                self.history_pos = self.history.len() - 1;
            }
        }
        
        pub fn go_back(&mut self) -> Option<String> {
            if self.history_pos > 0 {
                self.history_pos -= 1;
                Some(self.history[self.history_pos].clone())
            } else {
                None
            }
        }
        
        pub fn go_forward(&mut self) -> Option<String> {
            if self.history_pos + 1 < self.history.len() {
                self.history_pos += 1;
                Some(self.history[self.history_pos].clone())
            } else {
                None
            }
        }
    }
    
    /// Main entry point
    pub fn main() -> ! {
        // Initialize display
        let surface_id = match FabricDisplay::alloc_surface(SCREEN_WIDTH, SCREEN_HEIGHT) {
            Ok(id) => id,
            Err(_) => fatal_error("Display alloc failed"),
        };
        
        let mut buffer: Vec<u32> = vec![C_DARK_GRAY; (SCREEN_WIDTH * SCREEN_HEIGHT) as usize];
        let mut display = Display::new(surface_id, &mut buffer);
        let mut browser = Browser::new();
        
        // Initial load
        load_page(&mut browser, &mut display);
        
        // Main event loop
        loop {
            // Poll for keyboard input (non-blocking)
            match FabricKeyboard::read() {
                Key::None => {
                    // No key pressed, small delay to prevent busy-waiting
                    sleep_ms(5);
                }
                key => {
                    handle_key(key, &mut browser, &mut display);
                }
            }
            
            // Re-render if in viewing mode (for smooth scroll or animations)
            if browser.mode == BrowserMode::Viewing {
                display.render_browser(&browser);
                display.present();
            }
        }
    }
    
    /// Handle keyboard input based on current mode
    fn handle_key(key: Key, browser: &mut Browser, display: &mut Display) {
        match browser.mode {
            BrowserMode::Viewing => handle_viewing_key(key, browser, display),
            BrowserMode::UrlEditing => handle_url_editing_key(key, browser, display),
            BrowserMode::Loading => {
                // Ignore keys during loading, or allow cancel
                if key == Key::Escape {
                    browser.mode = BrowserMode::Viewing;
                }
            }
            BrowserMode::Error => {
                // Any key returns to viewing
                browser.mode = BrowserMode::Viewing;
                display.render_browser(browser);
                display.present();
            }
        }
    }
    
    /// Handle keys in viewing mode
    fn handle_viewing_key(key: Key, browser: &mut Browser, display: &mut Display) {
        match key {
            // Scrolling
            Key::Down => browser.scroll_down(1),
            Key::Up => browser.scroll_up(1),
            Key::PageDown => browser.page_down(),
            Key::PageUp => browser.page_up(),
            Key::Home => browser.scroll_offset = 0,
            Key::End => browser.scroll_offset = browser.content_lines.len().saturating_sub(MAX_LINES),
            
            // URL bar focus
            Key::Tab | Key::Ascii(b'l') | Key::Ascii(b'L') => {
                browser.mode = BrowserMode::UrlEditing;
                browser.url_buffer = browser.url.clone();
                browser.cursor_pos = browser.url_buffer.len();
                browser.status_message = String::from("Edit URL, press Enter to navigate");
            }
            
            // Navigation
            Key::Ascii(b'r') | Key::Ascii(b'R') => {
                // Reload
                load_page(browser, display);
            }
            Key::Ascii(b'b') | Key::Ascii(b'B') => {
                // Back
                if let Some(url) = browser.go_back() {
                    browser.navigate(&url);
                    load_page(browser, display);
                }
            }
            Key::Ascii(b'f') | Key::Ascii(b'F') => {
                // Forward
                if let Some(url) = browser.go_forward() {
                    browser.navigate(&url);
                    load_page(browser, display);
                }
            }
            
            _ => {}
        }
        
        display.render_browser(browser);
        display.present();
    }
    
    /// Handle keys in URL editing mode
    fn handle_url_editing_key(key: Key, browser: &mut Browser, display: &mut Display) {
        match key {
            Key::Enter => {
                // Navigate to URL
                let url = browser.url_buffer.clone();
                browser.navigate(&url);
                load_page(browser, display);
            }
            Key::Escape => {
                // Cancel editing
                browser.mode = BrowserMode::Viewing;
                browser.url_buffer = browser.url.clone();
                browser.status_message = String::from("Cancelled");
                display.render_browser(browser);
                display.present();
            }
            Key::Backspace => {
                if browser.cursor_pos > 0 {
                    browser.cursor_pos -= 1;
                    browser.url_buffer.remove(browser.cursor_pos);
                }
            }
            Key::Delete => {
                if browser.cursor_pos < browser.url_buffer.len() {
                    browser.url_buffer.remove(browser.cursor_pos);
                }
            }
            Key::Left => {
                if browser.cursor_pos > 0 {
                    browser.cursor_pos -= 1;
                }
            }
            Key::Right => {
                if browser.cursor_pos < browser.url_buffer.len() {
                    browser.cursor_pos += 1;
                }
            }
            Key::Home => browser.cursor_pos = 0,
            Key::End => browser.cursor_pos = browser.url_buffer.len(),
            Key::Ascii(c) => {
                if browser.url_buffer.len() < 256 {
                    browser.url_buffer.insert(browser.cursor_pos, c as char);
                    browser.cursor_pos += 1;
                }
            }
            _ => {}
        }
        
        display.render_browser(browser);
        display.present();
    }
    
    /// Load page content
    fn load_page(browser: &mut Browser, display: &mut Display) {
        browser.mode = BrowserMode::Loading;
        display.render_browser(browser);
        display.present();
        
        // Parse URL
        let (is_https, host, path) = match parse_url(&browser.url) {
            Some(parts) => parts,
            None => {
                show_error(browser, display, "Invalid URL", "Could not parse URL format");
                return;
            }
        };
        
        // Fetch based on protocol
        let response = if is_https {
            // Try TLS first
            match FabricTls::https_get(&host, &path) {
                Ok(data) => data,
                Err(TlsError::CertificateExpired) => {
                    show_error(browser, display, "TLS Error", "Certificate has expired");
                    return;
                }
                Err(TlsError::CertificateInvalid) => {
                    show_error(browser, display, "TLS Error", "Certificate is invalid or untrusted");
                    return;
                }
                Err(TlsError::HostnameMismatch) => {
                    show_error(browser, display, "TLS Error", "Certificate hostname mismatch");
                    return;
                }
                Err(TlsError::HandshakeFailed) => {
                    show_error(browser, display, "TLS Error", "Handshake failed - server may not support TLS");
                    return;
                }
                Err(_) => {
                    show_error(browser, display, "TLS Error", "Connection failed");
                    return;
                }
            }
        } else {
            // Plain HTTP
            match HttpClient::get(&host, &path) {
                Ok(data) => data,
                Err(e) => {
                    show_error(browser, display, "HTTP Error", &format!("{:?}", e));
                    return;
                }
            }
        };
        
        // Parse HTTP response
        let (status, body) = match parse_http_response(&response) {
            Some((code, body)) => {
                if code >= 400 {
                    show_error(browser, display, 
                        &format!("HTTP {}", code), 
                        &format!("Server returned error code {}", code));
                    return;
                }
                (code, body)
            }
            None => {
                // Treat entire response as body
                (200, String::from_utf8_lossy(&response).to_string())
            }
        };
        
        // Extract text content
        let text_content = extract_body_text(&body);
        
        // Update browser
        browser.set_content(&text_content);
        browser.add_to_history(&browser.url.clone());
        browser.mode = BrowserMode::Viewing;
        browser.status_message = format!("Loaded {} ({} bytes)", browser.url, response.len());
        
        // Try to extract title
        if let Some(title) = extract_title(&body) {
            browser.page_title = title;
        }
        
        display.render_browser(browser);
        display.present();
    }
    
    /// Show error page
    fn show_error(browser: &mut Browser, display: &mut Display, title: &str, detail: &str) {
        browser.mode = BrowserMode::Error;
        browser.page_title = title.to_string();
        browser.set_content(&format!("{}\n\n{}", title, detail));
        browser.status_message = format!("Error: {}", title);
        display.render_browser(browser);
        display.present();
    }
    
    /// Parse URL into (is_https, host, path)
    fn parse_url(url: &str) -> Option<(bool, String, String)> {
        let url_lower = url.to_lowercase();
        
        if url_lower.starts_with("https://") {
            let rest = &url[8..];
            let (host, path) = split_host_path(rest);
            Some((true, host, path))
        } else if url_lower.starts_with("http://") {
            let rest = &url[7..];
            let (host, path) = split_host_path(rest);
            Some((false, host, path))
        } else {
            // Assume http if no scheme
            let (host, path) = split_host_path(url);
            Some((false, host, path))
        }
    }
    
    fn split_host_path(url: &str) -> (String, String) {
        if let Some(slash_pos) = url.find('/') {
            (url[..slash_pos].to_string(), url[slash_pos..].to_string())
        } else {
            (url.to_string(), String::from("/"))
        }
    }
    
    /// Display structure
    pub struct Display<'a> {
        surface_id: u64,
        buffer: &'a mut [u32],
    }
    
    impl<'a> Display<'a> {
        pub fn new(surface_id: u64, buffer: &'a mut [u32]) -> Self {
            Self { surface_id, buffer }
        }
        
        pub fn clear(&mut self, color: u32) {
            for pixel in self.buffer.iter_mut() {
                *pixel = color;
            }
        }
        
        pub fn present(&self) {
            let _ = FabricDisplay::blit_surface(self.surface_id, self.buffer.as_ptr(), self.buffer.len() * 4);
            let _ = FabricDisplay::present_surface(self.surface_id);
        }
        
        /// Render complete browser UI
        pub fn render_browser(&mut self, browser: &Browser) {
            self.clear(C_DARK_GRAY);
            
            // Draw URL bar
            self.draw_url_bar(browser);
            
            // Draw content area
            match browser.mode {
                BrowserMode::Viewing | BrowserMode::Loading => {
                    self.draw_content(browser);
                }
                BrowserMode::Error => {
                    self.draw_error_content(browser);
                }
                BrowserMode::UrlEditing => {
                    self.draw_content(browser);
                }
            }
            
            // Draw status bar
            self.draw_status_bar(browser);
            
            // Draw scrollbar if needed
            if browser.content_lines.len() > MAX_LINES {
                self.draw_scrollbar(browser);
            }
        }
        
        fn draw_url_bar(&mut self, browser: &mut Browser) {
            // URL bar background
            let bg_color = match browser.mode {
                BrowserMode::UrlEditing => C_WARM_100,
                _ => C_MID_GRAY,
            };
            draw_rect(self.buffer, 0, 0, SCREEN_WIDTH, URL_BAR_HEIGHT, bg_color);
            
            // URL text
            let url_text = match browser.mode {
                BrowserMode::UrlEditing => &browser.url_buffer,
                _ => &browser.url,
            };
            
            let text_color = match browser.mode {
                BrowserMode::UrlEditing => C_BLACK,
                _ => C_WHITE,
            };
            
            // Draw URL with scheme highlighting
            draw_text(self.buffer, 10, 18, url_text, text_color);
            
            // Draw cursor in edit mode
            if browser.mode == BrowserMode::UrlEditing {
                let cursor_x = 10 + (browser.cursor_pos as u32 * CHAR_WIDTH);
                draw_rect(self.buffer, cursor_x, 16, 2, 18, C_INFO);
            }
            
            // Draw navigation buttons
            draw_text(self.buffer, SCREEN_WIDTH - 150, 18, "[B]ack", 
                if browser.history_pos > 0 { C_OFF_WHITE } else { C_LIGHT_GRAY });
            draw_text(self.buffer, SCREEN_WIDTH - 80, 18, "[R]eload", C_OFF_WHITE);
            
            // Separator line
            draw_rect(self.buffer, 0, URL_BAR_HEIGHT, SCREEN_WIDTH, 2, C_WARM_600);
        }
        
        fn draw_content(&mut self, browser: &Browser) {
            let visible_lines = browser.content_lines.iter()
                .skip(browser.scroll_offset)
                .take(MAX_LINES);
            
            let mut y = MARGIN_Y;
            for line in visible_lines {
                draw_text(self.buffer, MARGIN_X, y, line, C_OFF_WHITE);
                y += LINE_HEIGHT;
            }
            
            // Loading indicator
            if browser.mode == BrowserMode::Loading {
                let msg = "Loading...";
                let x = (SCREEN_WIDTH - (msg.len() as u32 * CHAR_WIDTH)) / 2;
                let y = SCREEN_HEIGHT / 2;
                draw_rect(self.buffer, x - 10, y - 10, (msg.len() as u32 * CHAR_WIDTH) + 20, 30, C_MID_GRAY);
                draw_text(self.buffer, x, y, msg, C_WARM_500);
            }
        }
        
        fn draw_error_content(&mut self, browser: &Browser) {
            // Error icon
            let cx = 100;
            let cy = 150;
            for i in 0..40 {
                draw_pixel(self.buffer, cx + i, cy + i, C_ERROR);
                draw_pixel(self.buffer, cx + 40 - i, cy + i, C_ERROR);
            }
            
            // Error title
            draw_text(self.buffer, 160, 140, &browser.page_title, C_WHITE);
            
            // Separator
            draw_rect(self.buffer, 50, 190, SCREEN_WIDTH - 100, 2, C_LIGHT_GRAY);
            
            // Content
            self.draw_content(browser);
        }
        
        fn draw_status_bar(&mut self, browser: &Browser) {
            let status_y = SCREEN_HEIGHT - STATUS_HEIGHT;
            
            // Status bar background
            draw_rect(self.buffer, 0, status_y, SCREEN_WIDTH, STATUS_HEIGHT, C_MID_GRAY);
            
            // Status text
            draw_text(self.buffer, 10, status_y + 6, &browser.status_message, C_OFF_WHITE);
            
            // Scroll position indicator
            if browser.content_lines.len() > MAX_LINES {
                let scroll_text = format!("{} / {} lines", 
                    browser.scroll_offset + 1, 
                    browser.content_lines.len());
                let text_width = scroll_text.len() as u32 * CHAR_WIDTH;
                draw_text(self.buffer, SCREEN_WIDTH - text_width - 10, status_y + 6, 
                    &scroll_text, C_OFF_WHITE);
            }
        }
        
        fn draw_scrollbar(&mut self, browser: &Browser) {
            let scrollbar_x = SCREEN_WIDTH - 12;
            let content_height = SCREEN_HEIGHT - MARGIN_Y - STATUS_HEIGHT - 10;
            
            // Track
            draw_rect(self.buffer, scrollbar_x, MARGIN_Y, 8, content_height, C_MID_GRAY);
            
            // Thumb
            let thumb_height = (MAX_LINES as u32 * content_height) / browser.content_lines.len() as u32;
            let max_scroll = browser.content_lines.len() - MAX_LINES;
            let thumb_y = if max_scroll > 0 {
                MARGIN_Y + (browser.scroll_offset as u32 * (content_height - thumb_height)) / max_scroll as u32
            } else {
                MARGIN_Y
            };
            
            draw_rect(self.buffer, scrollbar_x, thumb_y, 8, thumb_height.max(20), C_WARM_500);
        }
    }
    
    // HTML and text processing
    fn parse_http_response(data: &[u8]) -> Option<(u16, String)> {
        let header_end = data.windows(4)
            .position(|w| w == b"\r\n\r\n")
            .or_else(|| data.windows(2).position(|w| w == b"\n\n"))?;
        
        let body_start = if data[header_end..].starts_with(b"\r\n\r\n") {
            header_end + 4
        } else {
            header_end + 2
        };
        
        let header_text = core::str::from_utf8(&data[..header_end]).ok()?;
        let first_line = header_text.lines().next()?;
        let parts: Vec<&str> = first_line.split_whitespace().collect();
        
        if parts.len() < 2 {
            return None;
        }
        
        let status_code = parts[1].parse::<u16>().ok()?;
        let body = String::from_utf8_lossy(&data[body_start..]).to_string();
        
        Some((status_code, body))
    }
    
    fn extract_body_text(html: &str) -> String {
        let body_start = html.to_lowercase().find("<body");
        let body_end = html.to_lowercase().find("</body>");
        
        let content = match (body_start, body_end) {
            (Some(start), Some(end)) if end > start => {
                if let Some(tag_end) = html[start..].find('>') {
                    &html[start + tag_end + 1..end]
                } else {
                    html
                }
            }
            _ => html,
        };
        
        strip_tags(content)
    }
    
    fn extract_title(html: &str) -> Option<String> {
        let title_start = html.to_lowercase().find("<title>")?;
        let title_end = html[title_start..].to_lowercase().find("</title>")?;
        Some(html[title_start + 7..title_start + title_end].to_string())
    }
    
    fn strip_tags(html: &str) -> String {
        let mut result = String::with_capacity(html.len() / 2);
        let mut in_tag = false;
        let mut in_script = false;
        
        let mut chars = html.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '<' {
                let ahead: String = chars.clone().take(7).collect();
                let lower = ahead.to_lowercase();
                if lower.starts_with("script") || lower.starts_with("style") {
                    in_script = true;
                } else if lower.starts_with("/script") || lower.starts_with("/style") {
                    in_script = false;
                    while chars.next() != Some('>') {}
                    continue;
                }
                in_tag = true;
                continue;
            }
            
            if in_script {
                continue;
            }
            
            if ch == '>' && in_tag {
                in_tag = false;
                continue;
            }
            
            if !in_tag {
                if ch == '&' {
                    if chars.peek() == Some(&'l') {
                        chars.next();
                        if chars.next() == Some('t') && chars.next() == Some(';') {
                            result.push('<');
                            continue;
                        }
                    } else if chars.peek() == Some(&'g') {
                        chars.next();
                        if chars.next() == Some('t') && chars.next() == Some(';') {
                            result.push('>');
                            continue;
                        }
                    } else if chars.peek() == Some(&'a') {
                        chars.next();
                        if chars.next() == Some('m') && chars.next() == Some('p') && chars.next() == Some(';') {
                            result.push('&');
                            continue;
                        }
                    }
                    result.push('&');
                } else if ch.is_whitespace() {
                    if !result.ends_with(' ') && !result.is_empty() {
                        result.push(' ');
                    }
                } else {
                    result.push(ch);
                }
            }
        }
        
        result.trim().to_string()
    }
    
    fn wrap_text(text: &str, width: usize) -> Vec<String> {
        let mut lines = Vec::new();
        let mut current = String::new();
        
        for word in text.split_whitespace() {
            if current.is_empty() {
                if word.len() > width {
                    for chunk in word.as_bytes().chunks(width) {
                        lines.push(String::from_utf8_lossy(chunk).to_string());
                    }
                } else {
                    current.push_str(word);
                }
            } else if current.len() + 1 + word.len() <= width {
                current.push(' ');
                current.push_str(word);
            } else {
                lines.push(current);
                current = word.to_string();
            }
        }
        
        if !current.is_empty() {
            lines.push(current);
        }
        
        lines
    }
    
    // Font rendering
    static FONT_5X7: &[u8] = &[
        0x00,0x00,0x00,0x00,0x00, 0x00,0x00,0x5F,0x00,0x00, 0x00,0x07,0x00,0x07,0x00,
        0x14,0x7F,0x14,0x7F,0x14, 0x24,0x2A,0x7F,0x2A,0x12, 0x23,0x13,0x08,0x64,0x62,
        0x36,0x49,0x55,0x22,0x50, 0x00,0x05,0x03,0x00,0x00, 0x00,0x1C,0x22,0x41,0x00,
        0x00,0x41,0x22,0x1C,0x00, 0x08,0x2A,0x1C,0x2A,0x08, 0x08,0x08,0x3E,0x08,0x08,
        0x00,0x50,0x30,0x00,0x00, 0x08,0x08,0x08,0x08,0x08, 0x00,0x60,0x60,0x00,0x00,
        0x20,0x10,0x08,0x04,0x02, 0x3E,0x51,0x49,0x45,0x3E, 0x00,0x42,0x7F,0x40,0x00,
        0x42,0x61,0x51,0x49,0x46, 0x21,0x41,0x45,0x4B,0x31, 0x18,0x14,0x12,0x7F,0x10,
        0x27,0x45,0x45,0x45,0x39, 0x3C,0x4A,0x49,0x49,0x30, 0x01,0x71,0x09,0x05,0x03,
        0x36,0x49,0x49,0x49,0x36, 0x06,0x49,0x49,0x29,0x1E, 0x00,0x36,0x36,0x00,0x00,
        0x00,0x56,0x36,0x00,0x00, 0x00,0x08,0x14,0x22,0x41, 0x14,0x14,0x14,0x14,0x14,
        0x41,0x22,0x14,0x08,0x00, 0x02,0x01,0x51,0x09,0x06, 0x32,0x49,0x79,0x41,0x3E,
        0x7E,0x11,0x11,0x11,0x7E, 0x7F,0x49,0x49,0x49,0x36, 0x3E,0x41,0x41,0x41,0x22,
        0x7F,0x41,0x41,0x22,0x1C, 0x7F,0x49,0x49,0x49,0x41, 0x7F,0x09,0x09,0x01,0x01,
        0x3E,0x41,0x41,0x51,0x32, 0x7F,0x08,0x08,0x08,0x7F, 0x00,0x41,0x7F,0x41,0x00,
        0x20,0x40,0x41,0x3F,0x01, 0x7F,0x08,0x14,0x22,0x41, 0x7F,0x40,0x40,0x40,0x40,
        0x7F,0x02,0x04,0x02,0x7F, 0x7F,0x04,0x08,0x10,0x7F, 0x3E,0x41,0x41,0x41,0x3E,
        0x7F,0x09,0x09,0x09,0x06, 0x3E,0x41,0x51,0x21,0x5E, 0x7F,0x09,0x19,0x29,0x46,
        0x46,0x49,0x49,0x49,0x31, 0x01,0x01,0x7F,0x01,0x01, 0x3F,0x40,0x40,0x40,0x3F,
        0x1F,0x20,0x40,0x20,0x1F, 0x7F,0x20,0x18,0x20,0x7F, 0x63,0x14,0x08,0x14,0x63,
        0x03,0x04,0x78,0x04,0x03, 0x61,0x51,0x49,0x45,0x43, 0x00,0x00,0x7F,0x41,0x41,
        0x02,0x04,0x08,0x10,0x20, 0x41,0x41,0x7F,0x00,0x00, 0x04,0x02,0x01,0x02,0x04,
        0x40,0x40,0x40,0x40,0x40, 0x00,0x01,0x02,0x04,0x00, 0x20,0x54,0x54,0x54,0x78,
        0x7F,0x48,0x44,0x44,0x38, 0x38,0x44,0x44,0x44,0x20, 0x38,0x44,0x44,0x48,0x7F,
        0x38,0x54,0x54,0x54,0x18, 0x08,0x7E,0x09,0x01,0x02, 0x08,0x14,0x54,0x54,0x3C,
        0x7F,0x08,0x04,0x04,0x78, 0x00,0x44,0x7D,0x40,0x00, 0x20,0x40,0x44,0x3D,0x00,
        0x00,0x7F,0x10,0x28,0x44, 0x00,0x41,0x7F,0x40,0x00, 0x7C,0x04,0x18,0x04,0x78,
        0x7C,0x08,0x04,0x04,0x78, 0x38,0x44,0x44,0x44,0x38, 0x7C,0x14,0x14,0x14,0x08,
        0x08,0x14,0x14,0x18,0x7C, 0x7C,0x08,0x04,0x04,0x08, 0x48,0x54,0x54,0x54,0x20,
        0x04,0x3F,0x44,0x40,0x20, 0x3C,0x40,0x40,0x20,0x7C, 0x1C,0x20,0x40,0x20,0x1C,
        0x3C,0x40,0x30,0x40,0x3C, 0x44,0x28,0x10,0x28,0x44, 0x0C,0x50,0x50,0x50,0x3C,
        0x44,0x64,0x54,0x4C,0x44, 0x00,0x08,0x36,0x41,0x00, 0x00,0x00,0x7F,0x00,0x00,
        0x00,0x41,0x36,0x08,0x00, 0x08,0x08,0x2A,0x1C,0x08,
    ];
    
    fn draw_char(buffer: &mut [u32], x: u32, y: u32, c: char, color: u32) {
        let idx = if c as u32 >= 32 && c as u32 <= 126 {
            (c as u32 - 32) as usize * 5
        } else {
            0
        };
        
        if idx + 4 >= FONT_5X7.len() {
            return;
        }
        
        for col in 0..5 {
            let col_data = FONT_5X7[idx + col];
            for row in 0..7 {
                if (col_data >> row) & 1 != 0 {
                    let px = x + col as u32;
                    let py = y + row as u32;
                    if px < SCREEN_WIDTH && py < SCREEN_HEIGHT {
                        buffer[(py * SCREEN_WIDTH + px) as usize] = color;
                    }
                }
            }
        }
    }
    
    pub fn draw_text(buffer: &mut [u32], x: u32, y: u32, text: &str, color: u32) {
        let mut cx = x;
        for c in text.chars() {
            draw_char(buffer, cx, y, c, color);
            cx += 6;
        }
    }
    
    fn draw_pixel(buffer: &mut [u32], x: u32, y: u32, color: u32) {
        if x < SCREEN_WIDTH && y < SCREEN_HEIGHT {
            buffer[(y * SCREEN_WIDTH + x) as usize] = color;
        }
    }
    
    fn draw_rect(buffer: &mut [u32], x: u32, y: u32, w: u32, h: u32, color: u32) {
        for dy in 0..h {
            for dx in 0..w {
                let px = x + dx;
                let py = y + dy;
                if px < SCREEN_WIDTH && py < SCREEN_HEIGHT {
                    buffer[(py * SCREEN_WIDTH + px) as usize] = color;
                }
            }
        }
    }
    
    fn sleep_ms(ms: u32) {
        for _ in 0..ms * 10000 {
            unsafe { core::arch::asm!("nop"); }
        }
    }
    
    fn fatal_error(_msg: &str) -> ! {
        loop {
            unsafe { core::arch::asm!("hlt"); }
        }
    }
}

#[cfg(not(feature = "std"))]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    fabric_os::main()
}

#[cfg(not(feature = "std"))]
mod panic_handler {
    use core::panic::PanicInfo;
    
    #[panic_handler]
    fn panic(_info: &PanicInfo) -> ! {
        loop { unsafe { core::arch::asm!("hlt"); } }
    }
}
