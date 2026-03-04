//! OS Abstraction Layer
//!
//! This module provides platform-specific syscall wrappers.
//! For FabricOS, we use direct syscalls.
//! For other platforms, we fall back to libc/std implementations.

pub mod fabricsys;

pub use fabricsys::*;
