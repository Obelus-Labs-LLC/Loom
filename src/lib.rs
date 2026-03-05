//! Loom Browser Library
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod os;

#[cfg(feature = "std")]
pub use loom_core::*;
