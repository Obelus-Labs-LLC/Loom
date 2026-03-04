//! FabricOS Syscall Interface
//!
//! Direct syscall wrappers for FabricOS kernel.
//! These bypass libc and go straight to the kernel.

#![allow(dead_code, unused_imports)]

extern crate alloc;
use alloc::format;
use alloc::string::String;

/// Syscall numbers for FabricOS
/// 
/// FabricOS uses custom syscall numbers (not Linux compatible)
/// Phase 9: Socket syscalls = 10-17
/// Phase 10: Display syscalls = 18-20
#[repr(usize)]
#[derive(Debug, Clone, Copy)]
pub enum Syscall {
    // Socket syscalls (Phase 9) - FabricOS specific
    Socket = 10,
    Bind = 11,
    Listen = 12,
    Accept = 13,
    Connect = 14,
    Send = 15,
    Recv = 16,
    Shutdown = 17,
    
    // Display syscalls (Phase 10)
    DisplayAlloc = 18,    // Allocate a display surface
    DisplayBlit = 19,     // Blit buffer to surface
    DisplayPresent = 20,  // Present surface to screen
    
    // DNS syscall (Phase 11)
    DnsResolve = 22,      // Resolve hostname to IPv4
    
    // Poll syscall (Phase 12)
    Poll = 24,            // Poll for I/O events
    
    // File syscalls (to be implemented in FabricOS)
    Read = 100,
    Write = 101,
    Open = 102,
    Close = 103,
    
    // Process syscalls
    Exit = 200,
    Fork = 201,
    Exec = 202,
    Wait = 203,
    
    // Memory syscalls
    Brk = 300,
    Mmap = 301,
    Munmap = 302,
}

// Constants for external use
pub const SYS_SOCKET: u64 = 10;
pub const SYS_BIND: u64 = 11;
pub const SYS_LISTEN: u64 = 12;
pub const SYS_ACCEPT: u64 = 13;
pub const SYS_CONNECT: u64 = 14;
pub const SYS_SEND: u64 = 15;
pub const SYS_RECV: u64 = 16;
pub const SYS_SHUTDOWN: u64 = 17;
pub const SYS_DISPLAY_ALLOC: u64 = 18;
pub const SYS_DISPLAY_BLIT: u64 = 19;
pub const SYS_DISPLAY_PRESENT: u64 = 20;
pub const SYS_DNS_RESOLVE: u64 = 22;
pub const SYS_POLL: u64 = 24;

/// Poll events
pub const POLLIN: u16 = 0x01;
pub const POLLPRI: u16 = 0x02;
pub const POLLOUT: u16 = 0x04;
pub const POLLERR: u16 = 0x08;
pub const POLLHUP: u16 = 0x10;
pub const POLLNVAL: u16 = 0x20;

/// PollFd structure for poll() syscall
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PollFd {
    pub fd: u32,
    pub events: u16,
    pub revents: u16,
}

impl PollFd {
    pub fn new(fd: RawFd, events: u16) -> Self {
        Self {
            fd: fd.as_raw() as u32,
            events,
            revents: 0,
        }
    }
}

/// DNS resolution error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DnsError {
    Timeout = 1,
    NotFound = 2,
    KernelError = 3,
}

/// HTTP client error
#[derive(Debug, Clone)]
pub enum HttpError {
    ConnectFailed(i32),
    SendFailed(i32),
    RecvFailed(i32),
    DnsFailed(DnsError),
    SocketFailed(i32),
    PollFailed(i32),
    Timeout,
    ConnectionClosed,
}

/// Yield to kernel (syscall 0 or dedicated yield)
#[inline(always)]
pub fn yield_syscall() {
    unsafe {
        // Use a simple sleep/yield syscall or just nop
        // For now, use syscall 0 as yield
        core::arch::asm!(
            "syscall",
            in("rax") 0usize,
            out("rcx") _, out("r11") _,
            options(nostack, preserves_flags)
        );
    }
}

/// Sleep for approximately ms milliseconds (busy wait with yields)
pub fn sleep_ms(ms: u32) {
    for _ in 0..ms * 1000 {
        unsafe { core::arch::asm!("nop") };
    }
}

/// Raw syscall - x86_64 Linux ABI
/// rcx and r11 are clobbered by syscall instruction
#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn syscall1(n: usize, a1: usize) -> isize {
    let ret: isize;
    core::arch::asm!(
        "syscall",
        inlateout("rax") n => ret,
        in("rdi") a1,
        out("rcx") _, out("r11") _,
        options(nostack, preserves_flags)
    );
    ret
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn syscall2(n: usize, a1: usize, a2: usize) -> isize {
    let ret: isize;
    core::arch::asm!(
        "syscall",
        inlateout("rax") n => ret,
        in("rdi") a1,
        in("rsi") a2,
        out("rcx") _, out("r11") _,
        options(nostack, preserves_flags)
    );
    ret
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn syscall3(n: usize, a1: usize, a2: usize, a3: usize) -> isize {
    let ret: isize;
    core::arch::asm!(
        "syscall",
        inlateout("rax") n => ret,
        in("rdi") a1,
        in("rsi") a2,
        in("rdx") a3,
        out("rcx") _, out("r11") _,
        options(nostack, preserves_flags)
    );
    ret
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn syscall6(n: usize, a1: usize, a2: usize, a3: usize, a4: usize, a5: usize, a6: usize) -> isize {
    let ret: isize;
    core::arch::asm!(
        "syscall",
        inlateout("rax") n => ret,
        in("rdi") a1,
        in("rsi") a2,
        in("rdx") a3,
        in("r10") a4,
        in("r8") a5,
        in("r9") a6,
        out("rcx") _, out("r11") _,
        options(nostack, preserves_flags)
    );
    ret
}

/// Socket domain
#[repr(i32)]
#[derive(Debug, Clone, Copy)]
pub enum Domain {
    Inet = 2,   // AF_INET
    Inet6 = 10, // AF_INET6
}

/// Socket type
#[repr(i32)]
#[derive(Debug, Clone, Copy)]
pub enum SockType {
    Stream = 1, // SOCK_STREAM (TCP)
    Dgram = 2,  // SOCK_DGRAM (UDP)
}

/// Protocol
#[repr(i32)]
#[derive(Debug, Clone, Copy)]
pub enum Protocol {
    Ip = 0,  // Default
    Tcp = 6,
    Udp = 17,
}

/// Socket address for IPv4
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SockAddrIn {
    pub family: u16,    // AF_INET = 2
    pub port: u16,      // Port in network byte order
    pub addr: [u8; 4],  // IPv4 address
    pub padding: [u8; 8],
}

impl SockAddrIn {
    pub fn new(ip: [u8; 4], port: u16) -> Self {
        Self {
            family: 2, // AF_INET
            port: port.to_be(),
            addr: ip,
            padding: [0; 8],
        }
    }
}

/// Raw socket file descriptor
#[derive(Debug, Clone, Copy)]
pub struct RawFd(i32);

impl RawFd {
    pub fn as_raw(&self) -> i32 {
        self.0
    }
    
    pub fn is_valid(&self) -> bool {
        self.0 >= 0
    }
}

/// Socket operations for FabricOS
pub struct FabricSocket;

impl FabricSocket {
    /// Create a new socket
    /// Returns socket fd or negative error code
    pub fn socket(domain: Domain, sock_type: SockType, protocol: Protocol) -> Result<RawFd, i32> {
        let ret = unsafe {
            syscall3(
                Syscall::Socket as usize,
                domain as usize,
                sock_type as usize,
                protocol as usize,
            )
        };
        
        if ret < 0 {
            Err(-ret as i32)
        } else {
            Ok(RawFd(ret as i32))
        }
    }
    
    /// Bind socket to address
    pub fn bind(fd: RawFd, addr: &SockAddrIn) -> Result<(), i32> {
        let ret = unsafe {
            syscall3(
                Syscall::Bind as usize,
                fd.as_raw() as usize,
                addr as *const _ as usize,
                core::mem::size_of::<SockAddrIn>() as usize,
            )
        };
        
        if ret < 0 {
            Err(-ret as i32)
        } else {
            Ok(())
        }
    }
    
    /// Listen for incoming connections
    pub fn listen(fd: RawFd, backlog: i32) -> Result<(), i32> {
        let ret = unsafe {
            syscall2(
                Syscall::Listen as usize,
                fd.as_raw() as usize,
                backlog as usize,
            )
        };
        
        if ret < 0 {
            Err(-ret as i32)
        } else {
            Ok(())
        }
    }
    
    /// Accept incoming connection
    pub fn accept(fd: RawFd) -> Result<(RawFd, SockAddrIn), i32> {
        let mut addr: SockAddrIn = unsafe { core::mem::zeroed() };
        let mut addr_len = core::mem::size_of::<SockAddrIn>() as u32;
        
        let ret = unsafe {
            syscall3(
                Syscall::Accept as usize,
                fd.as_raw() as usize,
                &mut addr as *mut _ as usize,
                &mut addr_len as *mut _ as usize,
            )
        };
        
        if ret < 0 {
            Err(-ret as i32)
        } else {
            Ok((RawFd(ret as i32), addr))
        }
    }
    
    /// Connect to remote address
    pub fn connect(fd: RawFd, addr: &SockAddrIn) -> Result<(), i32> {
        let ret = unsafe {
            syscall3(
                Syscall::Connect as usize,
                fd.as_raw() as usize,
                addr as *const _ as usize,
                core::mem::size_of::<SockAddrIn>() as usize,
            )
        };
        
        if ret < 0 {
            Err(-ret as i32)
        } else {
            Ok(())
        }
    }
    
    /// Send data
    pub fn send(fd: RawFd, buf: &[u8], _flags: i32) -> Result<usize, i32> {
        let ret = unsafe {
            syscall3(
                Syscall::Send as usize,
                fd.as_raw() as usize,
                buf.as_ptr() as usize,
                buf.len() as usize,
            )
        };
        
        if ret < 0 {
            Err(-ret as i32)
        } else {
            Ok(ret as usize)
        }
    }
    
    /// Receive data
    pub fn recv(fd: RawFd, buf: &mut [u8], _flags: i32) -> Result<usize, i32> {
        let ret = unsafe {
            syscall3(
                Syscall::Recv as usize,
                fd.as_raw() as usize,
                buf.as_mut_ptr() as usize,
                buf.len() as usize,
            )
        };
        
        if ret < 0 {
            Err(-ret as i32)
        } else {
            Ok(ret as usize)
        }
    }
    
    /// Shutdown socket
    pub fn shutdown(fd: RawFd, how: i32) -> Result<(), i32> {
        let ret = unsafe {
            syscall2(
                Syscall::Shutdown as usize,
                fd.as_raw() as usize,
                how as usize,
            )
        };
        
        if ret < 0 {
            Err(-ret as i32)
        } else {
            Ok(())
        }
    }
    
    /// Close socket (uses Close syscall)
    pub fn close(fd: RawFd) -> Result<(), i32> {
        let ret = unsafe {
            syscall1(Syscall::Close as usize, fd.as_raw() as usize)
        };
        
        if ret < 0 {
            Err(-ret as i32)
        } else {
            Ok(())
        }
    }
}

/// Simple HTTP client using raw syscalls
pub struct SimpleHttp;

impl SimpleHttp {
    /// Perform a simple HTTP GET request
    /// Returns the raw response as a string
    pub fn get(host: &str, port: u16, path: &str) -> Result<String, String> {
        // Parse IP address (simple IPv4 parsing)
        let ip = Self::resolve_host(host)?;
        
        // Create socket
        let sock = FabricSocket::socket(Domain::Inet, SockType::Stream, Protocol::Tcp)
            .map_err(|e| format!("Socket failed: {}", e))?;
        
        // Connect
        let addr = SockAddrIn::new(ip, port);
        FabricSocket::connect(sock, &addr)
            .map_err(|e| format!("Connect failed: {}", e))?;
        
        // Build HTTP request
        let request = format!(
            "GET {} HTTP/1.1\r\nHost: {}\r\nUser-Agent: Loom/0.1\r\nConnection: close\r\n\r\n",
            path, host
        );
        
        // Send request
        FabricSocket::send(sock, request.as_bytes(), 0)
            .map_err(|e| format!("Send failed: {}", e))?;
        
        // Receive response
        let mut response = String::new();
        let mut buf = [0u8; 4096];
        
        loop {
            match FabricSocket::recv(sock, &mut buf, 0) {
                Ok(0) => break, // Connection closed
                Ok(n) => {
                    response.push_str(&String::from_utf8_lossy(&buf[..n]));
                }
                Err(e) => {
                    if e == 11 { // EAGAIN
                        continue;
                    }
                    return Err(format!("Recv failed: {}", e));
                }
            }
        }
        
        // Close socket
        let _ = FabricSocket::close(sock);
        
        Ok(response)
    }
    
    /// Simple host resolution (returns 93.184.216.34 for example.com)
    fn resolve_host(host: &str) -> Result<[u8; 4], String> {
        // TODO: Implement proper DNS resolution via syscall
        // For now, hardcode example.com
        match host {
            "example.com" => Ok([93, 184, 216, 34]),
            "93.184.216.34" => Ok([93, 184, 216, 34]),
            _ => Err(format!("Cannot resolve host: {}", host)),
        }
    }
}

/// Display syscalls for FabricOS Phase 10
/// 
/// These syscalls allow userspace to allocate display surfaces,
/// blit pixel data, and present to the screen.
pub struct FabricDisplay;

impl FabricDisplay {
    /// Allocate a display surface
    /// 
    /// # Arguments
    /// * `width` - Surface width in pixels
    /// * `height` - Surface height in pixels
    /// 
    /// # Returns
    /// Surface ID on success, negative error code on failure
    pub fn alloc_surface(width: u32, height: u32) -> Result<u64, i32> {
        let ret = unsafe {
            syscall2(
                Syscall::DisplayAlloc as usize,
                width as usize,
                height as usize,
            )
        };
        
        if ret < 0 {
            Err(-ret as i32)
        } else {
            Ok(ret as u64)
        }
    }
    
    /// Blit pixel data to a surface
    /// 
    /// # Arguments
    /// * `surface_id` - Surface ID from alloc_surface
    /// * `buffer` - Pixel data (BGRA format, 4 bytes per pixel)
    /// * `size` - Buffer size in bytes
    /// 
    /// # Returns
    /// Number of bytes blitted on success
    pub fn blit_surface(surface_id: u64, buffer: *const u32, size_bytes: usize) -> Result<usize, i32> {
        let ret = unsafe {
            syscall3(
                Syscall::DisplayBlit as usize,
                surface_id as usize,
                buffer as usize,
                size_bytes,
            )
        };
        
        if ret < 0 {
            Err(-ret as i32)
        } else {
            Ok(ret as usize)
        }
    }
    
    /// Present surface to screen
    /// 
    /// # Arguments
    /// * `surface_id` - Surface ID to present
    pub fn present_surface(surface_id: u64) -> Result<(), i32> {
        let ret = unsafe {
            syscall1(
                Syscall::DisplayPresent as usize,
                surface_id as usize,
            )
        };
        
        if ret < 0 {
            Err(-ret as i32)
        } else {
            Ok(())
        }
    }
    
    /// Free a display surface
    /// Note: This uses the Close syscall (103) for now
    pub fn free_surface(surface_id: u64) -> Result<(), i32> {
        let ret = unsafe {
            syscall1(
                Syscall::Close as usize,
                surface_id as usize,
            )
        };
        
        if ret < 0 {
            Err(-ret as i32)
        } else {
            Ok(())
        }
    }
}

/// DNS Resolution for FabricOS
pub struct FabricDns;

impl FabricDns {
    /// Resolve a hostname to IPv4 address using syscall 22
    /// 
    /// # Arguments
    /// * `hostname` - Hostname to resolve (e.g., "example.com")
    /// 
    /// # Returns
    /// Packed IPv4 address as big-endian u32 on success
    /// 
    /// # Errors
    /// Returns DnsError on failure with retry logic:
    /// - Retries 3x with 100ms delay on timeout
    pub fn resolve(hostname: &str) -> Result<u32, DnsError> {
        // Convert hostname to bytes with null terminator
        let host_bytes = hostname.as_bytes();
        let mut buf = [0u8; 256];
        if host_bytes.len() >= 255 {
            return Err(DnsError::KernelError);
        }
        buf[..host_bytes.len()].copy_from_slice(host_bytes);
        buf[host_bytes.len()] = 0; // Null terminator
        
        // Retry loop: 3 attempts with 100ms delay
        for attempt in 0..3 {
            let ret = unsafe {
                syscall2(
                    Syscall::DnsResolve as usize,
                    buf.as_ptr() as usize,
                    0, // flags
                )
            };
            
            if ret >= 0 {
                // Success: return packed IPv4
                return Ok(ret as u32);
            }
            
            let err = -ret as i32;
            match err {
                1 | 110 => { // ETIMEDOUT or EAGAIN - retry
                    if attempt < 2 {
                        sleep_ms(100);
                        continue;
                    }
                    return Err(DnsError::Timeout);
                }
                2 => return Err(DnsError::NotFound),
                _ => return Err(DnsError::KernelError),
            }
        }
        
        Err(DnsError::Timeout)
    }
    
    /// Convert packed u32 IP to byte array [a, b, c, d]
    pub fn ip_to_bytes(ip: u32) -> [u8; 4] {
        ip.to_be_bytes()
    }
}

/// Improved HTTP client for FabricOS
pub struct HttpClient;

impl HttpClient {
    /// Perform HTTP GET request and return raw bytes using poll() for efficient I/O
    /// 
    /// # Arguments
    /// * `ip` - Packed IPv4 address (big-endian u32)
    /// * `host` - Host header value (e.g., "example.com")
    /// * `path` - URL path (e.g., "/" or "/index.html")
    /// * `port` - Port number (usually 80 for HTTP)
    /// 
    /// # Returns
    /// Raw response bytes (headers + body)
    pub fn get_bytes(ip: u32, host: &str, path: &str, port: u16) -> Result<alloc::vec::Vec<u8>, HttpError> {
        use alloc::vec::Vec;
        const POLL_TIMEOUT_MS: i32 = 5000; // 5 second timeout per poll
        const MAX_RESPONSE_SIZE: usize = 1_048_576; // 1MB max
        
        // 1. Create socket
        let sock = FabricSocket::socket(Domain::Inet, SockType::Stream, Protocol::Tcp)
            .map_err(|e| HttpError::SocketFailed(e))?;
        
        // 2. Prepare address
        let ip_bytes = ip.to_be_bytes();
        let addr = SockAddrIn::new(ip_bytes, port);
        
        // 3. Connect
        FabricSocket::connect(sock, &addr)
            .map_err(|e| HttpError::ConnectFailed(e))?;
        
        // 4. Build and send HTTP request
        let request = format!(
            "GET {} HTTP/1.1\r\nHost: {}\r\nUser-Agent: Loom/0.1\r\nConnection: close\r\n\r\n",
            path, host
        );
        
        FabricSocket::send(sock, request.as_bytes(), 0)
            .map_err(|e| HttpError::SendFailed(e))?;
        
        // 5. Receive response using poll() for efficiency
        let mut response = Vec::new();
        let mut buf = [0u8; 4096];
        
        loop {
            // Poll for readability with timeout
            match FabricPoll::poll_readable(sock, POLL_TIMEOUT_MS) {
                Ok(true) => {
                    // Data available - recv it
                    match FabricSocket::recv(sock, &mut buf, 0) {
                        Ok(0) => {
                            // Connection closed gracefully
                            break;
                        }
                        Ok(n) => {
                            response.extend_from_slice(&buf[..n]);
                        }
                        Err(e) => {
                            let _ = FabricSocket::close(sock);
                            return Err(HttpError::RecvFailed(e));
                        }
                    }
                }
                Ok(false) => {
                    // POLLHUP - connection closed by server
                    break;
                }
                Err(HttpError::Timeout) => {
                    // No data within timeout - may be complete
                    // Try one more non-blocking recv to confirm
                    match FabricSocket::recv(sock, &mut buf, 0) {
                        Ok(0) => break, // Confirmed closed
                        Ok(n) => {
                            response.extend_from_slice(&buf[..n]);
                            continue; // More data, keep polling
                        }
                        Err(11) => break, // EAGAIN, assume done
                        Err(e) => {
                            let _ = FabricSocket::close(sock);
                            return Err(HttpError::RecvFailed(e));
                        }
                    }
                }
                Err(e) => {
                    let _ = FabricSocket::close(sock);
                    return Err(e);
                }
            }
            
            // Safety limit
            if response.len() > MAX_RESPONSE_SIZE {
                break;
            }
        }
        
        // 6. Close socket
        let _ = FabricSocket::close(sock);
        
        Ok(response)
    }
    
    /// Convenience: resolve host then fetch
    pub fn get(host: &str, path: &str) -> Result<Vec<u8>, HttpError> {
        let ip = FabricDns::resolve(host)
            .map_err(|e| HttpError::DnsFailed(e))?;
        Self::get_bytes(ip, host, path, 80)
    }
}

/// Poll-based I/O multiplexing for FabricOS
pub struct FabricPoll;

impl FabricPoll {
    /// Wait for events on file descriptors
    /// 
    /// # Arguments
    /// * `fds` - Array of PollFd structures (modified in-place by kernel)
    /// * `timeout_ms` - Timeout in milliseconds, -1 for infinite
    /// 
    /// # Returns
    /// Number of ready file descriptors on success
    pub fn poll(fds: &mut [PollFd], timeout_ms: i32) -> Result<usize, i32> {
        let ret = unsafe {
            syscall3(
                Syscall::Poll as usize,
                fds.as_mut_ptr() as usize,
                fds.len() as usize,
                timeout_ms as usize,
            )
        };
        
        if ret < 0 {
            Err(-ret as i32)
        } else {
            Ok(ret as usize)
        }
    }
    
    /// Poll a single fd for readability with timeout
    /// 
    /// # Returns
    /// - Ok(true): Data available (POLLIN)
    /// - Ok(false): Connection closed (POLLHUP)
    /// - Err(HttpError::Timeout): Timed out
    /// - Err(HttpError::PollFailed): Poll error
    pub fn poll_readable(fd: RawFd, timeout_ms: i32) -> Result<bool, HttpError> {
        let mut pollfd = PollFd::new(fd, POLLIN);
        
        match Self::poll(core::slice::from_mut(&mut pollfd), timeout_ms) {
            Ok(0) => Err(HttpError::Timeout),
            Ok(_) => {
                if pollfd.revents & POLLIN != 0 {
                    Ok(true) // Data available
                } else if pollfd.revents & POLLHUP != 0 {
                    Ok(false) // Connection closed
                } else if pollfd.revents & POLLERR != 0 {
                    Err(HttpError::PollFailed(POLLERR as i32))
                } else {
                    Err(HttpError::PollFailed(pollfd.revents as i32))
                }
            }
            Err(e) => Err(HttpError::PollFailed(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sockaddr_new() {
        let addr = SockAddrIn::new([127, 0, 0, 1], 8080);
        assert_eq!(addr.family, 2); // AF_INET
        assert_eq!(addr.addr, [127, 0, 0, 1]);
        assert_eq!(addr.port.to_be(), 8080);
    }
    
    #[test]
    fn test_http_request_format() {
        let request = format!(
            "GET {} HTTP/1.1\r\nHost: {}\r\nUser-Agent: Loom/0.1\r\nConnection: close\r\n\r\n",
            "/", "example.com"
        );
        assert!(request.contains("GET / HTTP/1.1"));
        assert!(request.contains("Host: example.com"));
    }
}
