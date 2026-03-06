//! Loom JS - JavaScript Engine
//!
//! Phase L23: V8 Integration
//! 
//! A JavaScript engine for the Loom browser supporting both:
//! - **Boa** (AI-Native mode): Pure Rust, minimal dependencies
//! - **V8** (Traditional mode): High performance with JIT compilation
//!
//! ## Features
//! - **Dual Engine**: Boa for AI-Native, V8 for Traditional mode
//! - **DOM Bindings**: JavaScript can read and modify the DOM tree
//! - **Event Handling**: onclick, onsubmit, and other events fire JavaScript
//! - **Basic APIs**: document, window, console.log, fetch stub
//! - **Sandbox**: JavaScript runs isolated with no filesystem access
//! - **GC Tuning**: Generational collection with frame-aware scheduling
//! - **Performance Metrics**: Execution time, memory usage, heap statistics
//!
//! ## Example
//! ```rust
//! use loom_js::{JsEngineFactory, BrowserMode};
//!
//! // Create V8 engine for Traditional mode
//! let mut engine = JsEngineFactory::create_for_mode(BrowserMode::Traditional).unwrap();
//! 
//! // Execute JavaScript
//! let result = engine.eval("1 + 1");
//! assert!(result.is_ok());
//! 
//! // Use console API
//! engine.eval("console.log('Hello, World!')");
//! 
//! // Get performance metrics
//! let metrics = engine.metrics();
//! println!("Execution time: {}ms", metrics.avg_execution_time_ms);
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

// Re-export main types
pub mod apis;
pub mod dom_bindings;
pub mod engine;
pub mod events;
pub mod sandbox;
pub mod gc_tuning;
pub mod engine_trait;

// Engine implementations
#[cfg(feature = "boa-engine")]
pub mod boa_impl;
#[cfg(feature = "v8-engine")]
pub mod v8_impl;
pub mod external_v8;
pub mod fabricos_v8;

// Re-export commonly used types
pub use engine::{JSEngine, JSEngineConfig, JSResult as LegacyJSResult};
pub use events::{EventManager, Event, EventType, EventHandler};
pub use sandbox::{Sandbox, SandboxConfig, SecurityPolicy, SandboxError};
pub use dom_bindings::{DomBridge, DomChangeType};
pub use gc_tuning::*;
pub use engine_trait::{
    JsEngine, JsEngineError, JsEngineFactory, JsException, JsMetrics,
    JsObject, JsResult, JsValue, MemoryPressure, HeapStatistics,
};

// Re-export engine implementations
#[cfg(feature = "boa-engine")]
pub use boa_impl::BoaJsEngine;
#[cfg(feature = "v8-engine")]
pub use v8_impl::V8Engine;
pub use external_v8::{ExternalV8Engine, CapabilityToken};
pub use fabricos_v8::{FabricOSV8Engine, init_v8_platform, shutdown_v8_platform};

/// Version of the JavaScript engine
pub const VERSION: &str = "0.3.0-L23";

/// Check if V8 JavaScript engine is available (Traditional mode)
pub fn is_v8_available() -> bool {
    #[cfg(feature = "v8-engine")]
    {
        true
    }
    #[cfg(not(feature = "v8-engine"))]
    {
        false
    }
}

/// Check if Boa JavaScript engine is available (AI-Native mode)
pub fn is_boa_available() -> bool {
    #[cfg(feature = "boa-engine")]
    {
        true
    }
    #[cfg(not(feature = "boa-engine"))]
    {
        false
    }
}

/// Check if external V8 process engine is available.
/// Always returns true — the external V8 engine is a no_std-safe
/// IPC proxy that compiles unconditionally. Actual availability
/// depends on whether the V8 process is running on FabricOS.
pub fn is_external_v8_available() -> bool {
    true
}

/// Get available engine names
pub fn available_engines() -> Vec<&'static str> {
    let mut engines = Vec::new();
    if is_boa_available() {
        engines.push("Boa");
    }
    if is_v8_available() {
        engines.push("V8");
    }
    if is_external_v8_available() {
        engines.push("V8 (external)");
    }
    engines
}

/// Create a new JavaScript engine with default configuration (legacy)
pub fn create_engine() -> anyhow::Result<JSEngine> {
    JSEngine::new()
}

/// Create a new JavaScript engine with strict sandboxing (legacy)
pub fn create_sandboxed_engine() -> anyhow::Result<JSEngine> {
    let config = JSEngineConfig {
        enable_console: true,
        enable_fetch: false, // Disable network in sandboxed mode
        max_execution_time_ms: 1000, // 1 second limit
        max_memory_bytes: 16 * 1024 * 1024, // 16 MB limit
        strict_mode: true,
    };
    
    JSEngine::with_config(config)
}

/// Performance benchmark helper
pub fn benchmark_engine(engine: &mut dyn JsEngine, script: &str) -> JsMetrics {
    engine.reset_metrics();
    
    // Run the script multiple times for averaging
    for _ in 0..10 {
        let _ = engine.eval(script);
    }
    
    engine.metrics()
}

/// Compare Boa and V8 performance
pub fn compare_engines(script: &str) -> Result<(JsMetrics, JsMetrics), JsEngineError> {
    #[cfg(feature = "boa-engine")]
    let mut boa_engine = BoaJsEngine::new();
    #[cfg(feature = "boa-engine")]
    boa_engine.initialize()?;
    
    #[cfg(feature = "v8-engine")]
    let mut v8_engine = V8Engine::new();
    #[cfg(feature = "v8-engine")]
    v8_engine.initialize()?;
    
    #[cfg(all(feature = "boa-engine", feature = "v8-engine"))]
    {
        let boa_metrics = benchmark_engine(&mut boa_engine, script);
        let v8_metrics = benchmark_engine(&mut v8_engine, script);
        Ok((boa_metrics, v8_metrics))
    }
    
    #[cfg(not(all(feature = "boa-engine", feature = "v8-engine")))]
    {
        Err(JsEngineError::InitializationFailed(
            "Both engines not available for comparison".to_string()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(VERSION, "0.3.0-L23");
    }

    #[test]
    fn test_available_engines() {
        let engines = available_engines();
        assert!(!engines.is_empty());
        
        if is_boa_available() {
            assert!(engines.contains(&"Boa"));
        }
        if is_v8_available() {
            assert!(engines.contains(&"V8"));
        }
    }

    #[test]
    fn test_js_value_creation() {
        let values = vec![
            JsValue::Undefined,
            JsValue::Null,
            JsValue::Bool(true),
            JsValue::Number(42.0),
            JsValue::String("hello".to_string()),
        ];
        
        assert_eq!(values.len(), 5);
    }

    #[test]
    fn test_js_result_ok() {
        let result = JsResult::ok(JsValue::Number(42.0));
        assert!(result.is_ok());
        assert!(!result.is_err());
    }

    #[test]
    fn test_js_result_err() {
        let result = JsResult::err("test error");
        assert!(!result.is_ok());
        assert!(result.is_err());
    }

    #[test]
    fn test_memory_pressure() {
        let pressures = vec![
            MemoryPressure::Normal,
            MemoryPressure::Moderate,
            MemoryPressure::Critical,
        ];
        
        assert_eq!(pressures.len(), 3);
    }

    #[test]
    fn test_js_engine_factory() {
        // Test that factory can create engines based on mode
        // Note: This may fail if features are not enabled
        
        #[cfg(feature = "boa-engine")]
        {
            let result = JsEngineFactory::create_ai_native();
            assert!(result.is_ok());
        }
        
        // Traditional mode may use V8 or fallback to Boa
        let result = JsEngineFactory::create_traditional();
        // Should succeed if at least one engine is available
        if is_boa_available() || is_v8_available() {
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_heap_statistics() {
        let stats = HeapStatistics::default();
        assert_eq!(stats.total_heap_size, 0);
        assert_eq!(stats.used_heap_size, 0);
    }

    #[test]
    fn test_js_metrics() {
        let metrics = JsMetrics::default();
        assert_eq!(metrics.scripts_executed, 0);
        assert_eq!(metrics.total_execution_time_ms, 0);
    }

    #[cfg(feature = "boa-engine")]
    #[test]
    fn test_boa_engine_basic() {
        use crate::boa_impl::BoaJsEngine;
        
        let mut engine = BoaJsEngine::new();
        assert!(engine.initialize().is_ok());
        
        let result = engine.eval("1 + 1");
        assert!(result.is_ok());
        
        let result = engine.eval("console.log('test')");
        assert!(result.is_ok());
    }

    #[cfg(feature = "v8-engine")]
    #[test]
    fn test_v8_engine_basic() {
        use crate::v8_impl::V8Engine;
        
        let mut engine = V8Engine::new();
        assert!(engine.initialize().is_ok());
        
        let result = engine.eval("1 + 1");
        assert!(result.is_ok());
        
        let result = engine.eval("console.log('test')");
        assert!(result.is_ok());
    }

    #[cfg(all(feature = "boa-engine", feature = "v8-engine"))]
    #[test]
    fn test_engine_comparison() {
        use crate::boa_impl::BoaJsEngine;
        use crate::v8_impl::V8Engine;
        
        let mut boa = BoaJsEngine::new();
        boa.initialize().unwrap();
        
        let mut v8 = V8Engine::new();
        v8.initialize().unwrap();
        
        // Test basic arithmetic on both engines
        let boa_result = boa.eval("2 + 2");
        let v8_result = v8.eval("2 + 2");
        
        assert!(boa_result.is_ok());
        assert!(v8_result.is_ok());
        
        // Both should return 4
        assert_eq!(boa_result.value, JsValue::Number(4.0));
        assert_eq!(v8_result.value, JsValue::Number(4.0));
    }

    #[test]
    fn test_js_object_creation() {
        let obj = JsObject {
            id: 123,
            prototype: None,
        };
        assert_eq!(obj.id, 123);
    }

    #[test]
    fn test_js_exception_creation() {
        let exc = JsException {
            message: "Test error".to_string(),
            stack_trace: Some("at line 10".to_string()),
            line: Some(10),
            column: Some(5),
        };
        
        assert_eq!(exc.message, "Test error");
        assert!(exc.stack_trace.is_some());
    }

    #[test]
    fn test_engine_error_display() {
        let err = JsEngineError::OutOfMemory;
        let msg = format!("{}", err);
        assert!(!msg.is_empty());
    }
}
