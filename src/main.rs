//! Loom Browser - Main Entry Point
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
// FABRICOS NO-STD IMPLEMENTATION
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
    
    // Temperature-aware accent colors (Warm theme - default)
    pub const C_WARM_50: u32 = 0xFFF7F6F5;
    pub const C_WARM_100: u32 = 0xFFF0EDE9;
    pub const C_WARM_500: u32 = 0xFFD4A574;
    pub const C_WARM_600: u32 = 0xFFB8935F;
    
    // Status colors
    pub const C_SUCCESS: u32 = 0xFF4CAF50;
    pub const C_WARNING: u32 = 0xFFFFA726;
    pub const C_ERROR: u32 = 0xFFE53935;
    pub const C_INFO: u32 = 0xFF42A5F5;
    
    // Display config
    pub const SCREEN_WIDTH: u32 = 1280;
    pub const SCREEN_HEIGHT: u32 = 800;
    
    // Text layout config
    pub const MARGIN_X: u32 = 20;
    pub const MARGIN_Y: u32 = 60; // Space for URL bar
    pub const LINE_HEIGHT: u32 = 16;
    pub const CHAR_WIDTH: u32 = 6;
    pub const CONTENT_WIDTH: u32 = SCREEN_WIDTH - 2 * MARGIN_X;
    pub const MAX_LINES: usize = ((SCREEN_HEIGHT - MARGIN_Y - 20) / LINE_HEIGHT) as usize;
    
    /// Main entry point for FabricOS
    pub fn main() -> ! {
        // Initialize display
        let surface_id = match FabricDisplay::alloc_surface(SCREEN_WIDTH, SCREEN_HEIGHT) {
            Ok(id) => id,
            Err(_) => fatal_error("Display alloc failed"),
        };
        
        let mut buffer: Vec<u32> = vec![C_DARK_GRAY; (SCREEN_WIDTH * SCREEN_HEIGHT) as usize];
        let mut display = Display::new(surface_id, &mut buffer);
        
        // Target URL (hardcoded for now)
        const TARGET_HOST: &str = "example.com";
        const TARGET_PATH: &str = "/";
        const TARGET_URL: &str = "http://example.com/";
        
        // Show initial loading screen
        display.clear(C_DARK_GRAY);
        display.draw_chrome(TARGET_URL, "Loading...");
        display.present();
        
        // Step 1: DNS Resolve
        display.set_status("Resolving DNS...");
        display.present();
        
        let ip = match FabricDns::resolve(TARGET_HOST) {
            Ok(ip) => {
                let ip_bytes = FabricDns::ip_to_bytes(ip);
                display.set_status(&format!("DNS: {}.{}.{}.{}", 
                    ip_bytes[0], ip_bytes[1], ip_bytes[2], ip_bytes[3]));
                display.present();
                sleep_ms(300);
                ip
            }
            Err(e) => {
                show_error_page(&mut display, &format!("DNS Error: {:?}", e),
                    "Could not resolve hostname. Check network connection.");
            }
        };
        
        // Step 2: HTTP GET
        display.set_status("Fetching content...");
        display.present();
        
        let response_bytes = match HttpClient::get_bytes(ip, TARGET_HOST, TARGET_PATH, 80) {
            Ok(bytes) => {
                display.set_status(&format!("Downloaded {} bytes", bytes.len()));
                display.present();
                sleep_ms(200);
                bytes
            }
            Err(HttpError::Timeout) => {
                show_error_page(&mut display, "Connection Timeout",
                    "The server did not respond in time.");
            }
            Err(HttpError::ConnectionClosed) => {
                show_error_page(&mut display, "Connection Closed",
                    "The server closed the connection unexpectedly.");
            }
            Err(e) => {
                show_error_page(&mut display, &format!("HTTP Error: {:?}", e),
                    "Failed to fetch the requested page.");
            }
        };
        
        // Step 3: Parse HTTP response
        display.set_status("Parsing response...");
        display.present();
        
        let (status_code, body) = match parse_http_response(&response_bytes) {
            Some((code, body)) => (code, body),
            None => {
                // Fallback: treat entire response as body
                (200, String::from_utf8_lossy(&response_bytes).to_string())
            }
        };
        
        if status_code >= 400 {
            show_error_page(&mut display, 
                &format!("HTTP {}", status_code),
                &format!("Server returned error code {}", status_code));
        }
        
        // Step 4: Extract text content from HTML
        display.set_status("Extracting content...");
        display.present();
        
        let text_content = extract_body_text(&body);
        if text_content.trim().is_empty() {
            show_error_page(&mut display, "Empty Content",
                "The page contains no readable text content.");
        }
        
        // Step 5: Render content
        display.set_status("Ready");
        display.render_content_page(TARGET_URL, &text_content);
        display.present();
        
        // Done - halt (no scroll for now)
        loop {
            unsafe { core::arch::asm!("hlt"); }
        }
    }
    
    /// Parse HTTP response, return (status_code, body_text)
    fn parse_http_response(data: &[u8]) -> Option<(u16, String)> {
        // Find header/body separator
        let header_end = data.windows(4)
            .position(|w| w == b"\r\n\r\n")
            .or_else(|| data.windows(2).position(|w| w == b"\n\n"))?;
        
        let body_start = if data[header_end..].starts_with(b"\r\n\r\n") {
            header_end + 4
        } else {
            header_end + 2
        };
        
        // Parse status line
        let header_text = core::str::from_utf8(&data[..header_end]).ok()?;
        let first_line = header_text.lines().next()?;
        let parts: Vec<&str> = first_line.split_whitespace().collect();
        if parts.len() < 2 {
            return None;
        }
        
        let status_code = parts[1].parse::<u16>().ok()?;
        
        // Extract body
        let body_bytes = &data[body_start..];
        let body_text = String::from_utf8_lossy(body_bytes).to_string();
        
        Some((status_code, body_text))
    }
    
    /// Extract text content from HTML
    fn extract_body_text(html: &str) -> String {
        // Find body tag
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
        
        strip_html_tags(content)
    }
    
    /// Strip HTML tags, preserve text
    fn strip_html_tags(html: &str) -> String {
        let mut result = String::with_capacity(html.len() / 2);
        let mut in_tag = false;
        let mut in_script = false;
        let mut chars = html.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '<' {
                // Check for script/style
                let ahead: String = chars.clone().take(7).collect();
                let lower = ahead.to_lowercase();
                if lower.starts_with("script") {
                    in_script = true;
                } else if lower.starts_with("style") {
                    in_script = true; // Reuse flag for style too
                } else if lower.starts_with("/script") || lower.starts_with("/style") {
                    in_script = false;
                    // Skip to >
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
                // Handle entities
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
                    } else if chars.peek() == Some(&'n') {
                        chars.next();
                        if chars.next() == Some('b') && chars.next() == Some('s') && 
                           chars.next() == Some('p') && chars.next() == Some(';') {
                            result.push(' ');
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
    
    /// Word wrap text to fit content width
    fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
        let mut lines = Vec::new();
        let mut current_line = String::new();
        
        for word in text.split_whitespace() {
            if current_line.is_empty() {
                // Word might be longer than max_chars
                if word.len() > max_chars {
                    // Split long word
                    for chunk in word.as_bytes().chunks(max_chars) {
                        lines.push(String::from_utf8_lossy(chunk).to_string());
                    }
                } else {
                    current_line.push_str(word);
                }
            } else if current_line.len() + 1 + word.len() <= max_chars {
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
    
    /// Display structure
    pub struct Display<'a> {
        surface_id: u64,
        buffer: &'a mut [u32],
        status_text: String,
    }
    
    impl<'a> Display<'a> {
        pub fn new(surface_id: u64, buffer: &'a mut [u32]) -> Self {
            Self {
                surface_id,
                buffer,
                status_text: String::new(),
            }
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
        
        pub fn set_status(&mut self, text: &str) {
            self.status_text = text.to_string();
        }
        
        /// Draw browser chrome (URL bar, status)
        pub fn draw_chrome(&mut self, url: &str, status: &str) {
            // URL bar background
            draw_rect(self.buffer, 0, 0, SCREEN_WIDTH, 40, C_MID_GRAY);
            
            // URL text
            draw_text(self.buffer, 10, 15, url, C_WHITE);
            
            // Status line background
            draw_rect(self.buffer, 0, SCREEN_HEIGHT - 24, SCREEN_WIDTH, 24, C_MID_GRAY);
            
            // Status text
            let status_to_show = if !self.status_text.is_empty() {
                &self.status_text
            } else {
                status
            };
            draw_text(self.buffer, 10, SCREEN_HEIGHT - 18, status_to_show, C_OFF_WHITE);
            
            // Separator line
            draw_rect(self.buffer, 0, 40, SCREEN_WIDTH, 2, C_WARM_600);
        }
        
        /// Render content page with text
        pub fn render_content_page(&mut self, url: &str, content: &str) {
            self.clear(C_DARK_GRAY);
            self.draw_chrome(url, "Ready");
            
            // Calculate content area
            let max_chars_per_line = (CONTENT_WIDTH / CHAR_WIDTH) as usize;
            let lines = wrap_text(content, max_chars_per_line);
            
            // Render visible lines
            let mut y = MARGIN_Y;
            for line in lines.iter().take(MAX_LINES) {
                draw_text(self.buffer, MARGIN_X, y, line, C_OFF_WHITE);
                y += LINE_HEIGHT;
            }
            
            // Show truncation notice if needed
            if lines.len() > MAX_LINES {
                let msg = format!("... {} more lines", lines.len() - MAX_LINES);
                draw_text(self.buffer, MARGIN_X, SCREEN_HEIGHT - 50, &msg, C_LIGHT_GRAY);
            }
        }
    }
    
    /// Show error page with design system styling
    fn show_error_page(display: &mut Display, error_title: &str, error_detail: &str) -> ! {
        display.clear(C_DARK_GRAY);
        
        // Error header bar
        draw_rect(display.buffer, 0, 0, SCREEN_WIDTH, 40, C_ERROR);
        draw_text(display.buffer, 10, 15, "Error", C_WHITE);
        
        // Error icon (simple X shape)
        let cx = 100;
        let cy = 120;
        for i in 0..40 {
            draw_pixel(display.buffer, cx + i, cy + i, C_ERROR);
            draw_pixel(display.buffer, cx + 40 - i, cy + i, C_ERROR);
        }
        
        // Error title
        draw_text(display.buffer, 160, 110, error_title, C_WHITE);
        
        // Separator
        draw_rect(display.buffer, 50, 160, SCREEN_WIDTH - 100, 2, C_LIGHT_GRAY);
        
        // Error detail
        let detail_lines = wrap_text(error_detail, 100);
        let mut y = 190;
        for line in detail_lines.iter().take(10) {
            draw_text(display.buffer, 50, y, line, C_OFF_WHITE);
            y += 20;
        }
        
        // Help text
        draw_text(display.buffer, 50, SCREEN_HEIGHT - 60, 
            "Press any key to retry (not implemented)", C_LIGHT_GRAY);
        
        display.present();
        
        // Halt
        loop {
            unsafe { core::arch::asm!("hlt"); }
        }
    }
    
    fn fatal_error(_msg: &str) -> ! {
        loop {
            unsafe { core::arch::asm!("hlt"); }
        }
    }
    
    fn sleep_ms(ms: u32) {
        for _ in 0..ms * 10000 {
            unsafe { core::arch::asm!("nop"); }
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
    
    // Font data and text rendering
    static FONT_5X7: &[u8] = &[
        0x00,0x00,0x00,0x00,0x00, /* Space */ 0x00,0x00,0x5F,0x00,0x00, /* ! */
        0x00,0x07,0x00,0x07,0x00, /* " */ 0x14,0x7F,0x14,0x7F,0x14, /* # */
        0x24,0x2A,0x7F,0x2A,0x12, /* $ */ 0x23,0x13,0x08,0x64,0x62, /* % */
        0x36,0x49,0x55,0x22,0x50, /* & */ 0x00,0x05,0x03,0x00,0x00, /* ' */
        0x00,0x1C,0x22,0x41,0x00, /* ( */ 0x00,0x41,0x22,0x1C,0x00, /* ) */
        0x08,0x2A,0x1C,0x2A,0x08, /* * */ 0x08,0x08,0x3E,0x08,0x08, /* + */
        0x00,0x50,0x30,0x00,0x00, /* , */ 0x08,0x08,0x08,0x08,0x08, /* - */
        0x00,0x60,0x60,0x00,0x00, /* . */ 0x20,0x10,0x08,0x04,0x02, /* / */
        0x3E,0x51,0x49,0x45,0x3E, /* 0 */ 0x00,0x42,0x7F,0x40,0x00, /* 1 */
        0x42,0x61,0x51,0x49,0x46, /* 2 */ 0x21,0x41,0x45,0x4B,0x31, /* 3 */
        0x18,0x14,0x12,0x7F,0x10, /* 4 */ 0x27,0x45,0x45,0x45,0x39, /* 5 */
        0x3C,0x4A,0x49,0x49,0x30, /* 6 */ 0x01,0x71,0x09,0x05,0x03, /* 7 */
        0x36,0x49,0x49,0x49,0x36, /* 8 */ 0x06,0x49,0x49,0x29,0x1E, /* 9 */
        0x00,0x36,0x36,0x00,0x00, /* : */ 0x00,0x56,0x36,0x00,0x00, /* ; */
        0x00,0x08,0x14,0x22,0x41, /* < */ 0x14,0x14,0x14,0x14,0x14, /* = */
        0x41,0x22,0x14,0x08,0x00, /* > */ 0x02,0x01,0x51,0x09,0x06, /* ? */
        0x32,0x49,0x79,0x41,0x3E, /* @ */ 0x7E,0x11,0x11,0x11,0x7E, /* A */
        0x7F,0x49,0x49,0x49,0x36, /* B */ 0x3E,0x41,0x41,0x41,0x22, /* C */
        0x7F,0x41,0x41,0x22,0x1C, /* D */ 0x7F,0x49,0x49,0x49,0x41, /* E */
        0x7F,0x09,0x09,0x01,0x01, /* F */ 0x3E,0x41,0x41,0x51,0x32, /* G */
        0x7F,0x08,0x08,0x08,0x7F, /* H */ 0x00,0x41,0x7F,0x41,0x00, /* I */
        0x20,0x40,0x41,0x3F,0x01, /* J */ 0x7F,0x08,0x14,0x22,0x41, /* K */
        0x7F,0x40,0x40,0x40,0x40, /* L */ 0x7F,0x02,0x04,0x02,0x7F, /* M */
        0x7F,0x04,0x08,0x10,0x7F, /* N */ 0x3E,0x41,0x41,0x41,0x3E, /* O */
        0x7F,0x09,0x09,0x09,0x06, /* P */ 0x3E,0x41,0x51,0x21,0x5E, /* Q */
        0x7F,0x09,0x19,0x29,0x46, /* R */ 0x46,0x49,0x49,0x49,0x31, /* S */
        0x01,0x01,0x7F,0x01,0x01, /* T */ 0x3F,0x40,0x40,0x40,0x3F, /* U */
        0x1F,0x20,0x40,0x20,0x1F, /* V */ 0x7F,0x20,0x18,0x20,0x7F, /* W */
        0x63,0x14,0x08,0x14,0x63, /* X */ 0x03,0x04,0x78,0x04,0x03, /* Y */
        0x61,0x51,0x49,0x45,0x43, /* Z */ 0x00,0x00,0x7F,0x41,0x41, /* [ */
        0x02,0x04,0x08,0x10,0x20, /* \ */ 0x41,0x41,0x7F,0x00,0x00, /* ] */
        0x04,0x02,0x01,0x02,0x04, /* ^ */ 0x40,0x40,0x40,0x40,0x40, /* _ */
        0x00,0x01,0x02,0x04,0x00, /* ` */ 0x20,0x54,0x54,0x54,0x78, /* a */
        0x7F,0x48,0x44,0x44,0x38, /* b */ 0x38,0x44,0x44,0x44,0x20, /* c */
        0x38,0x44,0x44,0x48,0x7F, /* d */ 0x38,0x54,0x54,0x54,0x18, /* e */
        0x08,0x7E,0x09,0x01,0x02, /* f */ 0x08,0x14,0x54,0x54,0x3C, /* g */
        0x7F,0x08,0x04,0x04,0x78, /* h */ 0x00,0x44,0x7D,0x40,0x00, /* i */
        0x20,0x40,0x44,0x3D,0x00, /* j */ 0x00,0x7F,0x10,0x28,0x44, /* k */
        0x00,0x41,0x7F,0x40,0x00, /* l */ 0x7C,0x04,0x18,0x04,0x78, /* m */
        0x7C,0x08,0x04,0x04,0x78, /* n */ 0x38,0x44,0x44,0x44,0x38, /* o */
        0x7C,0x14,0x14,0x14,0x08, /* p */ 0x08,0x14,0x14,0x18,0x7C, /* q */
        0x7C,0x08,0x04,0x04,0x08, /* r */ 0x48,0x54,0x54,0x54,0x20, /* s */
        0x04,0x3F,0x44,0x40,0x20, /* t */ 0x3C,0x40,0x40,0x20,0x7C, /* u */
        0x1C,0x20,0x40,0x20,0x1C, /* v */ 0x3C,0x40,0x30,0x40,0x3C, /* w */
        0x44,0x28,0x10,0x28,0x44, /* x */ 0x0C,0x50,0x50,0x50,0x3C, /* y */
        0x44,0x64,0x54,0x4C,0x44, /* z */ 0x00,0x08,0x36,0x41,0x00, /* { */
        0x00,0x00,0x7F,0x00,0x00, /* | */ 0x00,0x41,0x36,0x08,0x00, /* } */
        0x08,0x08,0x2A,0x1C,0x08, /* ~ */
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
