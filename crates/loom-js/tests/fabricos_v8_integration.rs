//! FabricOS V8 Integration Tests
//!
//! These tests validate the V8 JavaScript engine integration.
//!
//! # Running Tests
//!
//! Without libv8_shim.a (compilation only):
//! ```bash
//! cargo test --package loom-js --test fabricos_v8_integration
//! ```
//!
//! With libv8_shim.a (full integration):
//! ```bash
//! cargo test --package loom-js --test fabricos_v8_integration --features fabricos-v8
//! ```

use loom_js::fabricos_v8::{validate_compiles, diagnostic_info, VERSION};

/// Test that the module compiles correctly
#[test]
fn test_module_compiles() {
    assert!(validate_compiles(), "Module should compile without errors");
}

/// Test version constant
#[test]
fn test_version() {
    assert_eq!(VERSION, "0.1.0-L23");
}

/// Test diagnostic info output
#[test]
fn test_diagnostic_info() {
    let info = diagnostic_info();
    
    // Should contain key information
    assert!(info.contains("FabricOS V8"), "Should mention FabricOS V8");
    assert!(info.contains("libv8_shim.a"), "Should mention required library");
    assert!(info.contains("L13.7"), "Should reference L13.7");
}

/// Test that engine can be created (if available)
#[test]
fn test_engine_creation() {
    use loom_js::fabricos_v8::FabricOSV8Engine;
    
    match FabricOSV8Engine::new() {
        Ok(mut engine) => {
            println!("✅ V8 engine created successfully");
            
            // Try to initialize
            match engine.initialize() {
                Ok(_) => {
                    println!("✅ V8 engine initialized");
                    
                    // Cleanup
                    let _ = engine.shutdown();
                }
                Err(e) => {
                    println!("⚠️  Engine initialization failed (expected without lib): {:?}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠️  Engine creation failed (expected without lib): {:?}", e);
            // This is expected when libv8_shim.a is not available
        }
    }
}

/// Integration validation test (1+2=3)
/// 
/// This test is ignored by default because it requires libv8_shim.a.
/// Run with: cargo test -- --ignored
#[test]
#[ignore = "Requires libv8_shim.a from FabricOS L13.7"]
fn test_validate_integration() {
    use loom_js::fabricos_v8::validate_integration;
    
    validate_integration()
        .expect("V8 integration validation should pass with libv8_shim.a");
}

/// Test 1+2=3 specifically
#[test]
#[ignore = "Requires libv8_shim.a from FabricOS L13.7"]
fn test_one_plus_two_equals_three() {
    use loom_js::fabricos_v8::{FabricOSV8Engine, init_v8_platform, shutdown_v8_platform};
    use loom_js::engine_trait::{JsEngine, JsValue};
    
    unsafe {
        init_v8_platform().expect("Failed to init V8 platform");
    }
    
    let mut engine = FabricOSV8Engine::new().expect("Failed to create engine");
    engine.initialize().expect("Failed to initialize engine");
    
    let result = engine.eval("1 + 2").expect("Failed to execute script");
    
    match result.value {
        JsValue::Number(n) => {
            assert!((n - 3.0).abs() < 0.0001, "Expected 3, got {}", n);
        }
        other => panic!("Expected number, got {:?}", other),
    }
    
    engine.shutdown().expect("Failed to shutdown");
    shutdown_v8_platform();
}

/// Test console.log availability
#[test]
#[ignore = "Requires libv8_shim.a from FabricOS L13.7"]
fn test_console_log() {
    use loom_js::fabricos_v8::{FabricOSV8Engine, init_v8_platform, shutdown_v8_platform};
    use loom_js::engine_trait::JsEngine;
    
    unsafe {
        init_v8_platform().expect("Failed to init V8 platform");
    }
    
    let mut engine = FabricOSV8Engine::new().expect("Failed to create engine");
    engine.initialize().expect("Failed to initialize");
    
    // This should not panic
    let result = engine.eval("console.log('Hello from V8')");
    assert!(result.is_ok(), "console.log should be available");
    
    engine.shutdown().expect("Failed to shutdown");
    shutdown_v8_platform();
}

/// Test error handling
#[test]
#[ignore = "Requires libv8_shim.a from FabricOS L13.7"]
fn test_error_handling() {
    use loom_js::fabricos_v8::{FabricOSV8Engine, init_v8_platform, shutdown_v8_platform};
    use loom_js::engine_trait::JsEngine;
    
    unsafe {
        init_v8_platform().expect("Failed to init V8 platform");
    }
    
    let mut engine = FabricOSV8Engine::new().expect("Failed to create engine");
    engine.initialize().expect("Failed to initialize");
    
    // Syntax error
    let result = engine.eval("syntax error here");
    assert!(result.is_err(), "Syntax error should be caught");
    
    // Runtime error
    let result = engine.eval("throw new Error('test')");
    assert!(result.is_err(), "Thrown error should be caught");
    
    engine.shutdown().expect("Failed to shutdown");
    shutdown_v8_platform();
}

/// Performance smoke test
#[test]
#[ignore = "Requires libv8_shim.a from FabricOS L13.7"]
fn test_performance_smoke() {
    use loom_js::fabricos_v8::{FabricOSV8Engine, init_v8_platform, shutdown_v8_platform};
    use loom_js::engine_trait::JsEngine;
    use std::time::Instant;
    
    unsafe {
        init_v8_platform().expect("Failed to init V8 platform");
    }
    
    let mut engine = FabricOSV8Engine::new().expect("Failed to create engine");
    engine.initialize().expect("Failed to initialize");
    
    let start = Instant::now();
    
    // Execute 100 simple scripts
    for i in 0..100 {
        let result = engine.eval(&format!("{} + {}", i, i));
        assert!(result.is_ok(), "Script {} should execute", i);
    }
    
    let elapsed = start.elapsed();
    println!("100 scripts executed in {:?}", elapsed);
    
    // Should be reasonably fast (under 1 second for 100 simple scripts)
    assert!(elapsed.as_secs() < 1, "Should execute 100 scripts in under 1 second");
    
    engine.shutdown().expect("Failed to shutdown");
    shutdown_v8_platform();
}
