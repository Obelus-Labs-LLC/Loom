//! FabricOS Syscall Interface
//!
//! Direct syscall wrappers for FabricOS kernel.
//! These bypass libc and go straight to the kernel.

#![allow(dead_code, unused_imports)]

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

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
    
    // Keyboard syscall (Phase 10B)
    KbRead = 21,          // Read keyboard input
    
    // DNS syscall (Phase 11)
    DnsResolve = 22,      // Resolve hostname to IPv4
    
    // Poll syscall (Phase 12)
    Poll = 24,            // Poll for I/O events
    
    // TLS syscalls (Phase 15)
    TlsConnect = 25,      // TLS handshake
    TlsSend = 26,         // Send encrypted data
    TlsRecv = 27,         // Receive encrypted data
    TlsClose = 28,        // Close TLS session
    
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
pub const SYS_TLS_CONNECT: u64 = 25;
pub const SYS_TLS_SEND: u64 = 26;
pub const SYS_TLS_RECV: u64 = 27;
pub const SYS_TLS_CLOSE: u64 = 28;
pub const SYS_KB_READ: u64 = 21;

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
    /// FabricOS ABI: rdi = type (1=stream,2=dgram), rsi = protocol (6=tcp,17=udp)
    pub fn socket(_domain: Domain, sock_type: SockType, protocol: Protocol) -> Result<RawFd, i32> {
        let ret = unsafe {
            syscall2(
                Syscall::Socket as usize,
                sock_type as usize,   // rdi = socket type
                protocol as usize,    // rsi = protocol
            )
        };

        if ret < 0 {
            Err(-ret as i32)
        } else {
            Ok(RawFd(ret as i32))
        }
    }
    
    /// Bind socket to address
    /// FabricOS ABI: rdi = fd, rsi = ip_u32 (big-endian), rdx = port
    pub fn bind(fd: RawFd, addr: &SockAddrIn) -> Result<(), i32> {
        let ip_u32 = ((addr.addr[0] as usize) << 24)
            | ((addr.addr[1] as usize) << 16)
            | ((addr.addr[2] as usize) << 8)
            | (addr.addr[3] as usize);
        let port = u16::from_be(addr.port) as usize;

        let ret = unsafe {
            syscall3(
                Syscall::Bind as usize,
                fd.as_raw() as usize,
                ip_u32,
                port,
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
    /// FabricOS ABI: rdi = fd, rsi = ip_u32 (big-endian), rdx = port
    pub fn connect(fd: RawFd, addr: &SockAddrIn) -> Result<(), i32> {
        // Pack IPv4 as big-endian u32 for kernel ABI
        let ip_u32 = ((addr.addr[0] as usize) << 24)
            | ((addr.addr[1] as usize) << 16)
            | ((addr.addr[2] as usize) << 8)
            | (addr.addr[3] as usize);
        let port = u16::from_be(addr.port) as usize;

        let ret = unsafe {
            syscall3(
                Syscall::Connect as usize,
                fd.as_raw() as usize,
                ip_u32,
                port,
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
    
    /// Close socket — uses shutdown syscall (17) since FabricOS
    /// doesn't have a separate socket close path
    pub fn close(fd: RawFd) -> Result<(), i32> {
        Self::shutdown(fd, 2) // SHUT_RDWR
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
                    host_bytes.len(), // hostname length (kernel expects rsi=name_len)
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

/// TLS/SSL support for FabricOS (syscalls 25-28)
pub struct FabricTls;

/// TLS session handle (returned by kernel)
pub type TlsSession = u32;

/// TLS-specific errors
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TlsError {
    HandshakeFailed = 1,
    CertificateInvalid = 2,
    CertificateExpired = 3,
    HostnameMismatch = 4,
    ConnectionClosed = 5,
    SendFailed = 6,
    RecvFailed = 7,
    KernelError = 8,
}

impl FabricTls {
    /// Establish TLS connection over existing socket
    /// 
    /// # Arguments
    /// * `sock` - Connected socket from FabricSocket
    /// * `hostname` - Server hostname for certificate validation
    /// 
    /// # Returns
    /// TLS session handle on success
    pub fn connect(sock: RawFd, hostname: &str) -> Result<TlsSession, TlsError> {
        // Validate hostname length
        if hostname.len() > 256 {
            return Err(TlsError::HostnameMismatch);
        }
        
        let ret = unsafe {
            syscall3(
                Syscall::TlsConnect as usize,
                sock.as_raw() as usize,
                hostname.as_ptr() as usize,
                hostname.len() as usize,
            )
        };
        
        if ret < 0 {
            let err = -ret as i32;
            match err {
                1 => Err(TlsError::HandshakeFailed),
                2 => Err(TlsError::CertificateInvalid),
                3 => Err(TlsError::CertificateExpired),
                4 => Err(TlsError::HostnameMismatch),
                _ => Err(TlsError::KernelError),
            }
        } else {
            Ok(ret as u32)
        }
    }
    
    /// Send encrypted data over TLS connection
    pub fn send(session: TlsSession, data: &[u8]) -> Result<usize, TlsError> {
        if data.len() > 65536 {
            return Err(TlsError::KernelError);
        }
        
        let ret = unsafe {
            syscall3(
                Syscall::TlsSend as usize,
                session as usize,
                data.as_ptr() as usize,
                data.len() as usize,
            )
        };
        
        if ret < 0 {
            let err = -ret as i32;
            match err {
                5 => Err(TlsError::ConnectionClosed),
                _ => Err(TlsError::SendFailed),
            }
        } else {
            Ok(ret as usize)
        }
    }
    
    /// Receive decrypted data from TLS connection
    pub fn recv(session: TlsSession, buf: &mut [u8]) -> Result<usize, TlsError> {
        let ret = unsafe {
            syscall3(
                Syscall::TlsRecv as usize,
                session as usize,
                buf.as_mut_ptr() as usize,
                buf.len() as usize,
            )
        };
        
        if ret < 0 {
            let err = -ret as i32;
            match err {
                5 => Err(TlsError::ConnectionClosed),
                _ => Err(TlsError::RecvFailed),
            }
        } else {
            Ok(ret as usize)
        }
    }
    
    /// Close TLS session
    pub fn close(session: TlsSession) -> Result<(), TlsError> {
        let ret = unsafe {
            syscall1(
                Syscall::TlsClose as usize,
                session as usize,
            )
        };
        
        if ret < 0 {
            Err(TlsError::KernelError)
        } else {
            Ok(())
        }
    }
    
    /// Perform HTTPS GET request
    pub fn https_get(host: &str, path: &str) -> Result<Vec<u8>, TlsError> {
        use alloc::vec::Vec;
        
        // 1. Resolve DNS
        let ip = FabricDns::resolve(host)
            .map_err(|_| TlsError::KernelError)?;
        
        // 2. Create socket
        let sock = FabricSocket::socket(Domain::Inet, SockType::Stream, Protocol::Tcp)
            .map_err(|_| TlsError::KernelError)?;
        
        // 3. Connect TCP
        let ip_bytes = FabricDns::ip_to_bytes(ip);
        let addr = SockAddrIn::new(ip_bytes, 443);
        FabricSocket::connect(sock, &addr)
            .map_err(|_| TlsError::KernelError)?;
        
        // 4. Start TLS handshake
        let session = Self::connect(sock, host)?;
        
        // 5. Send HTTP request over TLS
        let request = format!(
            "GET {} HTTP/1.1\r\nHost: {}\r\nUser-Agent: Loom/0.1\r\nConnection: close\r\n\r\n",
            path, host
        );
        
        Self::send(session, request.as_bytes())?;
        
        // 6. Receive response with timeout handling
        let mut response = Vec::new();
        let mut buf = [0u8; 4096];
        let mut empty_reads = 0;
        
        loop {
            match Self::recv(session, &mut buf) {
                Ok(0) => {
                    empty_reads += 1;
                    if empty_reads > 5 {
                        break;
                    }
                    sleep_ms(10);
                }
                Ok(n) => {
                    response.extend_from_slice(&buf[..n]);
                    empty_reads = 0;
                }
                Err(TlsError::ConnectionClosed) => break,
                Err(_) => break,
            }
            
            // Safety limit
            if response.len() > 1_048_576 {
                break;
            }
        }
        
        // 7. Cleanup
        let _ = Self::close(session);
        let _ = FabricSocket::close(sock);
        
        Ok(response)
    }
}

/// Keyboard input support (syscall 21)
pub struct FabricKeyboard;

/// Key event
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Key {
    Ascii(u8),
    Up,
    Down,
    Left,
    Right,
    PageUp,
    PageDown,
    Home,
    End,
    Enter,
    Escape,
    Tab,
    Backspace,
    Delete,
    None,
}

/// Keyboard scancode mapping (simplified PC/AT set 1)
impl FabricKeyboard {
    /// Read key from keyboard buffer (non-blocking)
    /// Returns Key::None if no key available
    pub fn read() -> Key {
        let ret = unsafe {
            syscall1(Syscall::KbRead as usize, 0)
        };
        
        if ret <= 0 {
            return Key::None;
        }
        
        let scancode = ret as u8;
        Self::scancode_to_key(scancode)
    }
    
    /// Poll for key with timeout (ms)
    pub fn poll_key(timeout_ms: u32) -> Key {
        let start = get_tick();
        loop {
            let key = Self::read();
            if key != Key::None {
                return key;
            }
            if get_tick() - start > timeout_ms as u64 {
                return Key::None;
            }
            sleep_ms(1);
        }
    }
    
    /// Convert scancode to key
    fn scancode_to_key(scancode: u8) -> Key {
        // Simple scancode to ASCII mapping (no shift handling for now)
        match scancode {
            0x01 => Key::Escape,
            0x0E => Key::Backspace,
            0x0F => Key::Tab,
            0x1C => Key::Enter,
            0x48 => Key::Up,
            0x50 => Key::Down,
            0x4B => Key::Left,
            0x4D => Key::Right,
            0x49 => Key::PageUp,
            0x51 => Key::PageDown,
            0x47 => Key::Home,
            0x4F => Key::End,
            0x53 => Key::Delete,
            // ASCII letters (assuming lowercase)
            0x1E => Key::Ascii(b'a'),
            0x30 => Key::Ascii(b'b'),
            0x2E => Key::Ascii(b'c'),
            0x20 => Key::Ascii(b'd'),
            0x12 => Key::Ascii(b'e'),
            0x21 => Key::Ascii(b'f'),
            0x22 => Key::Ascii(b'g'),
            0x23 => Key::Ascii(b'h'),
            0x17 => Key::Ascii(b'i'),
            0x24 => Key::Ascii(b'j'),
            0x25 => Key::Ascii(b'k'),
            0x26 => Key::Ascii(b'l'),
            0x32 => Key::Ascii(b'm'),
            0x31 => Key::Ascii(b'n'),
            0x18 => Key::Ascii(b'o'),
            0x19 => Key::Ascii(b'p'),
            0x10 => Key::Ascii(b'q'),
            0x13 => Key::Ascii(b'r'),
            0x1F => Key::Ascii(b's'),
            0x14 => Key::Ascii(b't'),
            0x16 => Key::Ascii(b'u'),
            0x2F => Key::Ascii(b'v'),
            0x11 => Key::Ascii(b'w'),
            0x2D => Key::Ascii(b'x'),
            0x15 => Key::Ascii(b'y'),
            0x2C => Key::Ascii(b'z'),
            // Numbers
            0x02 => Key::Ascii(b'1'),
            0x03 => Key::Ascii(b'2'),
            0x04 => Key::Ascii(b'3'),
            0x05 => Key::Ascii(b'4'),
            0x06 => Key::Ascii(b'5'),
            0x07 => Key::Ascii(b'6'),
            0x08 => Key::Ascii(b'7'),
            0x09 => Key::Ascii(b'8'),
            0x0A => Key::Ascii(b'9'),
            0x0B => Key::Ascii(b'0'),
            // Punctuation
            0x39 => Key::Ascii(b' '),
            0x33 => Key::Ascii(b','),
            0x34 => Key::Ascii(b'.'),
            0x35 => Key::Ascii(b'/'),
            0x0C => Key::Ascii(b'-'),
            0x0D => Key::Ascii(b'='),
            0x1A => Key::Ascii(b'['),
            0x1B => Key::Ascii(b']'),
            0x2B => Key::Ascii(b'\\'),
            0x27 => Key::Ascii(b';'),
            0x28 => Key::Ascii(b'\''),
            0x29 => Key::Ascii(b'`'),
            _ => Key::None,
        }
    }
}

/// Simple timer/tick counter (approximate)
static mut TICK_COUNT: u64 = 0;

pub fn get_tick() -> u64 {
    unsafe { TICK_COUNT }
}

pub fn increment_tick() {
    unsafe { TICK_COUNT += 1; }
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
