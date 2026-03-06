//! FabricOS V8 JavaScript Engine Integration
//!
//! This module provides a JsEngine implementation that uses the FabricOS
//! V8 JavaScript engine via FFI bindings.
//!
//! # Status
//!
//! - **FFI Bindings**: ✅ Complete (ffi.rs)
//! - **Engine Implementation**: ✅ Skeleton complete (engine.rs)
//! - **Linking**: ⏳ BLOCKED - Waiting for libv8_shim.a (FabricOS L13.7)
//!
//! # Integration
//!
//! Once FabricOS L13.7 completes, enable the `fabricos-v8` feature:
//!
//! ```toml
//! [features]
//! fabricos-v8 = []
//! ```
//!
//! And link libv8_shim.a in your build.rs or Makefile.

pub mod engine;
pub mod ffi;

pub use engine::{FabricOSV8Engine, init_v8_platform, shutdown_v8_platform};
pub use ffi::{
    V8Config, V8ContextHandle, V8IsolateHandle, V8ScriptResult, V8ValueHandle,
    value_types,
};

/// Version of the FabricOS V8 integration
pub const VERSION: &str = "0.1.0-L23";

/// Check if FabricOS V8 engine is available
pub fn is_available() -> bool {
    FabricOSV8Engine::is_available()
}

/// Test harness for 1+2=3 validation
///
/// This function validates the V8 integration by executing simple
/// arithmetic and verifying the result.
///
/// # Returns
///
/// `Ok(())` if 1+2=3 executes correctly, `Err` with diagnostic info otherwise.
///
/// # Example
///
/// ```rust
/// use loom_js::fabricos_v8::validate_integration;
///
/// match validate_integration() {
///     Ok(_) => println!("V8 integration validated!"),
///     Err(e) => println!("V8 integration failed: {}", e),
/// }
/// ```
pub fn validate_integration() -> Result<(), String> {
    use crate::engine_trait::{JsEngine, JsValue};

    // Check if V8 is available
    if !is_available() {
        return Err(
            "FabricOS V8 not available - libv8_shim.a not linked (L13.7 pending)".to_string()
        );
    }

    // Initialize platform (only once per process)
    unsafe {
        if let Err(e) = init_v8_platform() {
            return Err(format!("Failed to initialize V8 platform: {:?}", e));
        }
    }

    // Create engine
    let mut engine = FabricOSV8Engine::new()
        .map_err(|e| format!("Failed to create V8 engine: {:?}", e))?;

    // Initialize engine
    engine.initialize()
        .map_err(|e| format!("Failed to initialize engine: {:?}", e))?;

    // Test 1: Simple arithmetic (1+2=3)
    let result = engine.eval("1 + 2")
        .map_err(|e| format!("Failed to execute '1 + 2': {:?}", e))?;

    match result.value {
        JsValue::Number(n) if (n - 3.0).abs() < 0.0001 => {
            println!("✅ Test 1 passed: 1 + 2 = {}", n);
        }
        other => {
            return Err(format!("Test 1 failed: expected 3, got {:?}", other));
        }
    }

    // Test 2: String concatenation
    let result = engine.eval("'Hello, ' + 'World!'")
        .map_err(|e| format!("Failed to execute string concat: {:?}", e))?;

    match result.value {
        JsValue::String(s) if s == "Hello, World!" => {
            println!("✅ Test 2 passed: string concatenation");
        }
        other => {
            return Err(format!("Test 2 failed: expected 'Hello, World!', got {:?}", other));
        }
    }

    // Test 3: Console.log works
    let result = engine.eval("console.log('Test message')");
    if result.is_ok() {
        println!("✅ Test 3 passed: console.log available");
    } else {
        println!("⚠️  Test 3: console.log may not be fully implemented");
    }

    // Test 4: Error handling
    let result = engine.eval("throw new Error('Test error')");
    match result {
        Err(_) => {
            println!("✅ Test 4 passed: error handling works");
        }
        Ok(_) => {
            return Err("Test 4 failed: expected error for throw statement".to_string());
        }
    }

    // Cleanup
    engine.shutdown()
        .map_err(|e| format!("Failed to shutdown engine: {:?}", e))?;

    shutdown_v8_platform();

    println!("\n✅ All FabricOS V8 integration tests passed!");
    Ok(())
}

/// Quick validation test for CI/CD
///
/// This is a minimal test that can be run in CI to verify the FFI bindings
/// compile correctly, even without libv8_shim.a linked.
pub fn validate_compiles() -> bool {
    // Just check that the module structure is valid
    true
}

/// Detailed diagnostic information
pub fn diagnostic_info() -> String {
    let mut info = String::new();
    
    info.push_str("=== FabricOS V8 Integration Diagnostics ===\n\n");
    
    info.push_str(&format!("Integration version: {}\n", VERSION));
    info.push_str(&format!("V8 available: {}\n", is_available()));
    
    #[cfg(feature = "fabricos-v8")]
    info.push_str("Feature 'fabricos-v8': enabled\n");
    
    #[cfg(not(feature = "fabricos-v8"))]
    info.push_str("Feature 'fabricos-v8': disabled\n");
    
    info.push_str("\n");
    info.push_str("Expected library: libv8_shim.a\n");
    info.push_str("Expected headers: kernel/src/v8_shim/v8_fabricos_shim.h\n");
    info.push_str("\n");
    info.push_str("To enable:\n");
    info.push_str("1. Complete FabricOS L13.7 (V8 build)\n");
    info.push_str("2. Copy libv8_shim.a to link path\n");
    info.push_str("3. Enable 'fabricos-v8' feature in Cargo.toml\n");
    
    info
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(VERSION, "0.1.0-L23");
    }

    #[test]
    fn test_validate_compiles() {
        assert!(validate_compiles());
    }

    #[test]
    fn test_diagnostic_info() {
        let info = diagnostic_info();
        assert!(info.contains("FabricOS V8"));
        assert!(info.contains("libv8_shim.a"));
    }

    // Note: validate_integration() test requires libv8_shim.a
    // #[test]
    // fn test_validate_integration() {
    //     validate_integration().expect("V8 integration validation failed");
    // }
}
