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
            heap: UnsafeCell::new(0x400000 as *mut u8), // Start at 4MB (above code at 2MB)
            end: UnsafeCell::new(0xC00000 as *mut u8), // End at 12MB (8MB heap, mapped by kernel)
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
    
    // Initialize display via syscalls
    let width = 1280u32;
    let height = 800u32;
    
    // Allocate display surface (syscall 18)
    let surface_id = match FabricDisplay::alloc_surface(width, height) {
        Ok(id) => id,
        Err(_) => {
            // Fatal: can't allocate display
            loop { unsafe { core::arch::asm!("hlt"); } }
        }
    };
    
    // Create back buffer
    let mut buffer: Vec<u32> = vec![0; (width * height) as usize];
    
    // Color constants (BGRA format)
    let warm_gray: u32 = 0xFFF7F6F5;
    let red: u32 = 0xFF0000FF;
    let green: u32 = 0xFF00FF00;
    let blue: u32 = 0xFFFF0000;
    let yellow: u32 = 0xFF00FFFF;
    
    // Animation frame counter
    let mut frame: u32 = 0;
    
    loop {
        frame += 1;
        
        // Clear to warm gray
        for pixel in buffer.iter_mut() {
            *pixel = warm_gray;
        }
        
        // Draw colored rectangles
        draw_rect(&mut buffer, width, height, 100, 100, 200, 100, red);
        draw_rect(&mut buffer, width, height, 350, 100, 200, 100, green);
        draw_rect(&mut buffer, width, height, 600, 100, 200, 100, blue);
        
        // Draw animated rectangle
        let x = 100 + (frame % 200);
        draw_rect(&mut buffer, width, height, x, 400, 50, 50, yellow);
        
        // Test socket syscall - draw indicator
        match FabricSocket::socket(Domain::Inet, SockType::Stream, Protocol::Tcp) {
            Ok(fd) => {
                // Success - green indicator
                draw_rect(&mut buffer, width, height, width - 50, 10, 20, 20, green);
                let _ = FabricSocket::close(fd);
            }
            Err(_) => {
                // Failed - red indicator  
                draw_rect(&mut buffer, width, height, width - 50, 10, 20, 20, red);
            }
        }
        
        // Blit buffer to surface (syscall 19)
        let _ = FabricDisplay::blit_surface(
            surface_id,
            buffer.as_ptr(),
            buffer.len() * 4
        );
        
        // Present to screen (syscall 20)
        let _ = FabricDisplay::present_surface(surface_id);
        
        // Simple delay
        for _ in 0..500000 {
            unsafe { core::arch::asm!("nop"); }
        }
    }
}

#[cfg(not(feature = "std"))]
fn draw_rect(buffer: &mut [u32], width: u32, _height: u32, x: u32, y: u32, w: u32, h: u32, color: u32) {
    for dy in 0..h {
        for dx in 0..w {
            let px = x + dx;
            let py = y + dy;
            if px < width && py < _height {
                let offset = (py * width + px) as usize;
                buffer[offset] = color;
            }
        }
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
