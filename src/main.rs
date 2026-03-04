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
            heap: UnsafeCell::new(0x800000 as *mut u8), // Start at 8MB (above any ELF code)
            end: UnsafeCell::new(0x1000000 as *mut u8), // End at 16MB (8MB heap, mapped by kernel)
        }
    }
}

#[cfg(not(feature = "std"))]
unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let heap = *self.heap.get();
        let align = layout.align();
        let size = layout.size();
        
        // Align the pointer
        let aligned = ((heap as usize + align - 1) & !(align - 1)) as *mut u8;
        let new_heap = aligned.add(size);
        
        if new_heap > *self.end.get() {
            return core::ptr::null_mut();
        }
        
        *self.heap.get() = new_heap;
        aligned
    }
    
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Bump allocator doesn't free
    }
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

#[cfg(not(feature = "std"))]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    use os::fabricsys::*;
    use alloc::vec;
    use alloc::vec::Vec;
    use alloc::string::String;
    
    // Display constants
    let width = 1280u32;
    let height = 800u32;
    
    // Color constants (BGRA format)
    const COLOR_BLACK: u32 = 0xFF000000;
    const COLOR_DARK_GRAY: u32 = 0xFF222222;
    const COLOR_RED: u32 = 0xFF0000FF;
    const COLOR_WHITE: u32 = 0xFFFFFFFF;
    const COLOR_GREEN: u32 = 0xFF00FF00;
    const COLOR_YELLOW: u32 = 0xFF00FFFF;
    
    // Initialize display
    let surface_id = match FabricDisplay::alloc_surface(width, height) {
        Ok(id) => id,
        Err(_) => fatal_error("Display alloc failed"),
    };
    
    let mut buffer: Vec<u32> = vec![0; (width * height) as usize];
    
    // Show boot screen
    clear_screen(&mut buffer, COLOR_DARK_GRAY);
    draw_text(&mut buffer, width, 10, 10, "Loom HTTP Fetch Test", COLOR_WHITE);
    draw_text(&mut buffer, width, 10, 30, "Step 1: DNS Resolve...", COLOR_YELLOW);
    blit_and_present(surface_id, &buffer);
    
    // Hardcoded target
    const TARGET_HOST: &str = "example.com";
    const TARGET_PATH: &str = "/";
    
    // Step 1: DNS Resolve
    sleep_ms(500); // Visual delay
    
    let ip = match FabricDns::resolve(TARGET_HOST) {
        Ok(ip) => {
            let ip_bytes = FabricDns::ip_to_bytes(ip);
            let msg = format!("DNS OK: {}.{}.{}.{}", 
                ip_bytes[0], ip_bytes[1], ip_bytes[2], ip_bytes[3]);
            draw_text(&mut buffer, width, 10, 50, &msg, COLOR_GREEN);
            blit_and_present(surface_id, &buffer);
            ip
        }
        Err(e) => {
            let msg = format!("DNS FAILED: {:?}", e);
            show_error_screen(surface_id, &mut buffer, width, &msg);
        }
    };
    
    sleep_ms(200);
    
    // Step 2: HTTP GET
    draw_text(&mut buffer, width, 10, 70, "Step 2: HTTP GET...", COLOR_YELLOW);
    blit_and_present(surface_id, &buffer);
    
    let response = match HttpClient::get_bytes(ip, TARGET_HOST, TARGET_PATH, 80) {
        Ok(bytes) => {
            let msg = format!("HTTP OK: {} bytes", bytes.len());
            draw_text(&mut buffer, width, 10, 90, &msg, COLOR_GREEN);
            blit_and_present(surface_id, &buffer);
            bytes
        }
        Err(e) => {
            let msg = format!("HTTP FAILED: {:?}", e);
            show_error_screen(surface_id, &mut buffer, width, &msg);
        }
    };
    
    sleep_ms(200);
    
    // Step 3: Render response
    draw_text(&mut buffer, width, 10, 110, "Step 3: Rendering...", COLOR_YELLOW);
    blit_and_present(surface_id, &buffer);
    sleep_ms(200);
    
    // Clear to dark gray and render the response
    clear_screen(&mut buffer, COLOR_DARK_GRAY);
    
    // Draw header
    draw_text(&mut buffer, width, 10, 10, "=== HTTP RESPONSE ===", COLOR_GREEN);
    draw_text(&mut buffer, width, 10, 30, TARGET_HOST, COLOR_WHITE);
    
    // Render first 500 bytes as hex dump
    let display_len = response.len().min(500);
    let mut y = 60;
    let mut x = 10;
    
    for (i, byte) in response[..display_len].iter().enumerate() {
        // Draw hex value
        let hex = byte_to_hex(*byte);
        draw_text(&mut buffer, width, x, y, &hex, COLOR_WHITE);
        
        x += 30;
        if x > width - 40 || *byte == b'\n' {
            x = 10;
            y += 20;
            if y > height - 30 {
                break; // Screen full
            }
        }
        
        // Add space between every 8 bytes
        if (i + 1) % 8 == 0 && x > 10 {
            x += 10;
        }
    }
    
    // Show truncation notice if needed
    if response.len() > 500 {
        let msg = format!("... ({} more bytes)", response.len() - 500);
        y += 20;
        draw_text(&mut buffer, width, 10, y, &msg, COLOR_YELLOW);
    }
    
    blit_and_present(surface_id, &buffer);
    
    // Done - halt forever
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

#[cfg(not(feature = "std"))]
fn fatal_error(msg: &str) -> ! {
    // Can't display, just halt
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

#[cfg(not(feature = "std"))]
fn show_error_screen(surface_id: u64, buffer: &mut [u32], width: u32, msg: &str) -> ! {
    use os::fabricsys::*;
    
    const COLOR_RED: u32 = 0xFF0000FF;
    const COLOR_WHITE: u32 = 0xFFFFFFFF;
    
    // Clear to red
    for pixel in buffer.iter_mut() {
        *pixel = COLOR_RED;
    }
    
    // Draw error message
    draw_text(buffer, width, 10, 10, "ERROR:", COLOR_WHITE);
    draw_text(buffer, width, 10, 40, msg, COLOR_WHITE);
    draw_text(buffer, width, 10, 100, "System Halted", COLOR_WHITE);
    
    // Present
    let _ = FabricDisplay::blit_surface(surface_id, buffer.as_ptr(), buffer.len() * 4);
    let _ = FabricDisplay::present_surface(surface_id);
    
    // Halt forever
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

#[cfg(not(feature = "std"))]
fn clear_screen(buffer: &mut [u32], color: u32) {
    for pixel in buffer.iter_mut() {
        *pixel = color;
    }
}

#[cfg(not(feature = "std"))]
fn blit_and_present(surface_id: u64, buffer: &[u32]) {
    use os::fabricsys::*;
    let _ = FabricDisplay::blit_surface(surface_id, buffer.as_ptr(), buffer.len() * 4);
    let _ = FabricDisplay::present_surface(surface_id);
}

#[cfg(not(feature = "std"))]
fn sleep_ms(ms: u32) {
    for _ in 0..ms * 10000 {
        unsafe { core::arch::asm!("nop"); }
    }
}

/// Convert a byte to hex string (e.g., 0x4A -> "4A")
#[cfg(not(feature = "std"))]
fn byte_to_hex(byte: u8) -> alloc::string::String {
    use alloc::string::String;
    const HEX_CHARS: &[u8] = b"0123456789ABCDEF";
    let mut result = String::with_capacity(2);
    result.push(HEX_CHARS[(byte >> 4) as usize] as char);
    result.push(HEX_CHARS[(byte & 0xF) as usize] as char);
    result
}

/// Simple bitmap font (5x7) for ASCII 32-127
#[cfg(not(feature = "std"))]
static FONT_5X7: &[u8] = &[
    // Space (32)
    0x00, 0x00, 0x00, 0x00, 0x00,
    // ! (33)
    0x00, 0x00, 0x5F, 0x00, 0x00,
    // " (34)
    0x00, 0x07, 0x00, 0x07, 0x00,
    // # (35)
    0x14, 0x7F, 0x14, 0x7F, 0x14,
    // $ (36)
    0x24, 0x2A, 0x7F, 0x2A, 0x12,
    // % (37)
    0x23, 0x13, 0x08, 0x64, 0x62,
    // & (38)
    0x36, 0x49, 0x55, 0x22, 0x50,
    // ' (39)
    0x00, 0x05, 0x03, 0x00, 0x00,
    // ( (40)
    0x00, 0x1C, 0x22, 0x41, 0x00,
    // ) (41)
    0x00, 0x41, 0x22, 0x1C, 0x00,
    // * (42)
    0x08, 0x2A, 0x1C, 0x2A, 0x08,
    // + (43)
    0x08, 0x08, 0x3E, 0x08, 0x08,
    // , (44)
    0x00, 0x50, 0x30, 0x00, 0x00,
    // - (45)
    0x08, 0x08, 0x08, 0x08, 0x08,
    // . (46)
    0x00, 0x60, 0x60, 0x00, 0x00,
    // / (47)
    0x20, 0x10, 0x08, 0x04, 0x02,
    // 0 (48)
    0x3E, 0x51, 0x49, 0x45, 0x3E,
    // 1 (49)
    0x00, 0x42, 0x7F, 0x40, 0x00,
    // 2 (50)
    0x42, 0x61, 0x51, 0x49, 0x46,
    // 3 (51)
    0x21, 0x41, 0x45, 0x4B, 0x31,
    // 4 (52)
    0x18, 0x14, 0x12, 0x7F, 0x10,
    // 5 (53)
    0x27, 0x45, 0x45, 0x45, 0x39,
    // 6 (54)
    0x3C, 0x4A, 0x49, 0x49, 0x30,
    // 7 (55)
    0x01, 0x71, 0x09, 0x05, 0x03,
    // 8 (56)
    0x36, 0x49, 0x49, 0x49, 0x36,
    // 9 (57)
    0x06, 0x49, 0x49, 0x29, 0x1E,
    // : (58)
    0x00, 0x36, 0x36, 0x00, 0x00,
    // ; (59)
    0x00, 0x56, 0x36, 0x00, 0x00,
    // < (60)
    0x00, 0x08, 0x14, 0x22, 0x41,
    // = (61)
    0x14, 0x14, 0x14, 0x14, 0x14,
    // > (62)
    0x41, 0x22, 0x14, 0x08, 0x00,
    // ? (63)
    0x02, 0x01, 0x51, 0x09, 0x06,
    // @ (64)
    0x32, 0x49, 0x79, 0x41, 0x3E,
    // A (65)
    0x7E, 0x11, 0x11, 0x11, 0x7E,
    // B (66)
    0x7F, 0x49, 0x49, 0x49, 0x36,
    // C (67)
    0x3E, 0x41, 0x41, 0x41, 0x22,
    // D (68)
    0x7F, 0x41, 0x41, 0x22, 0x1C,
    // E (69)
    0x7F, 0x49, 0x49, 0x49, 0x41,
    // F (70)
    0x7F, 0x09, 0x09, 0x01, 0x01,
    // G (71)
    0x3E, 0x41, 0x41, 0x51, 0x32,
    // H (72)
    0x7F, 0x08, 0x08, 0x08, 0x7F,
    // I (73)
    0x00, 0x41, 0x7F, 0x41, 0x00,
    // J (74)
    0x20, 0x40, 0x41, 0x3F, 0x01,
    // K (75)
    0x7F, 0x08, 0x14, 0x22, 0x41,
    // L (76)
    0x7F, 0x40, 0x40, 0x40, 0x40,
    // M (77)
    0x7F, 0x02, 0x04, 0x02, 0x7F,
    // N (78)
    0x7F, 0x04, 0x08, 0x10, 0x7F,
    // O (79)
    0x3E, 0x41, 0x41, 0x41, 0x3E,
    // P (80)
    0x7F, 0x09, 0x09, 0x09, 0x06,
    // Q (81)
    0x3E, 0x41, 0x51, 0x21, 0x5E,
    // R (82)
    0x7F, 0x09, 0x19, 0x29, 0x46,
    // S (83)
    0x46, 0x49, 0x49, 0x49, 0x31,
    // T (84)
    0x01, 0x01, 0x7F, 0x01, 0x01,
    // U (85)
    0x3F, 0x40, 0x40, 0x40, 0x3F,
    // V (86)
    0x1F, 0x20, 0x40, 0x20, 0x1F,
    // W (87)
    0x7F, 0x20, 0x18, 0x20, 0x7F,
    // X (88)
    0x63, 0x14, 0x08, 0x14, 0x63,
    // Y (89)
    0x03, 0x04, 0x78, 0x04, 0x03,
    // Z (90)
    0x61, 0x51, 0x49, 0x45, 0x43,
    // [ (91)
    0x00, 0x00, 0x7F, 0x41, 0x41,
    // \ (92)
    0x02, 0x04, 0x08, 0x10, 0x20,
    // ] (93)
    0x41, 0x41, 0x7F, 0x00, 0x00,
    // ^ (94)
    0x04, 0x02, 0x01, 0x02, 0x04,
    // _ (95)
    0x40, 0x40, 0x40, 0x40, 0x40,
    // ` (96)
    0x00, 0x01, 0x02, 0x04, 0x00,
    // a (97)
    0x20, 0x54, 0x54, 0x54, 0x78,
    // b (98)
    0x7F, 0x48, 0x44, 0x44, 0x38,
    // c (99)
    0x38, 0x44, 0x44, 0x44, 0x20,
    // d (100)
    0x38, 0x44, 0x44, 0x48, 0x7F,
    // e (101)
    0x38, 0x54, 0x54, 0x54, 0x18,
    // f (102)
    0x08, 0x7E, 0x09, 0x01, 0x02,
    // g (103)
    0x08, 0x14, 0x54, 0x54, 0x3C,
    // h (104)
    0x7F, 0x08, 0x04, 0x04, 0x78,
    // i (105)
    0x00, 0x44, 0x7D, 0x40, 0x00,
    // j (106)
    0x20, 0x40, 0x44, 0x3D, 0x00,
    // k (107)
    0x00, 0x7F, 0x10, 0x28, 0x44,
    // l (108)
    0x00, 0x41, 0x7F, 0x40, 0x00,
    // m (109)
    0x7C, 0x04, 0x18, 0x04, 0x78,
    // n (110)
    0x7C, 0x08, 0x04, 0x04, 0x78,
    // o (111)
    0x38, 0x44, 0x44, 0x44, 0x38,
    // p (112)
    0x7C, 0x14, 0x14, 0x14, 0x08,
    // q (113)
    0x08, 0x14, 0x14, 0x18, 0x7C,
    // r (114)
    0x7C, 0x08, 0x04, 0x04, 0x08,
    // s (115)
    0x48, 0x54, 0x54, 0x54, 0x20,
    // t (116)
    0x04, 0x3F, 0x44, 0x40, 0x20,
    // u (117)
    0x3C, 0x40, 0x40, 0x20, 0x7C,
    // v (118)
    0x1C, 0x20, 0x40, 0x20, 0x1C,
    // w (119)
    0x3C, 0x40, 0x30, 0x40, 0x3C,
    // x (120)
    0x44, 0x28, 0x10, 0x28, 0x44,
    // y (121)
    0x0C, 0x50, 0x50, 0x50, 0x3C,
    // z (122)
    0x44, 0x64, 0x54, 0x4C, 0x44,
    // { (123)
    0x00, 0x08, 0x36, 0x41, 0x00,
    // | (124)
    0x00, 0x00, 0x7F, 0x00, 0x00,
    // } (125)
    0x00, 0x41, 0x36, 0x08, 0x00,
    // ~ (126)
    0x08, 0x08, 0x2A, 0x1C, 0x08,
];

/// Draw a single character using 5x7 bitmap font
#[cfg(not(feature = "std"))]
fn draw_char(buffer: &mut [u32], screen_width: u32, x: u32, y: u32, c: char, color: u32) {
    let idx = if c as u32 >= 32 && c as u32 <= 126 {
        (c as u32 - 32) as usize * 5
    } else {
        0 // Space for unknown chars
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
                if px < screen_width && py < 800 {
                    let offset = (py * screen_width + px) as usize;
                    buffer[offset] = color;
                }
            }
        }
    }
}

/// Draw a string at (x, y)
#[cfg(not(feature = "std"))]
fn draw_text(buffer: &mut [u32], screen_width: u32, x: u32, y: u32, text: &str, color: u32) {
    let mut cx = x;
    for c in text.chars() {
        draw_char(buffer, screen_width, cx, y, c, color);
        cx += 6; // 5px char + 1px spacing
    }
}

#[cfg(not(feature = "std"))]
mod panic_handler {
    use core::panic::PanicInfo;
    
    #[panic_handler]
    fn panic(_info: &PanicInfo) -> ! {
        loop { unsafe { core::arch::asm!("hlt"); } }
    }
}
