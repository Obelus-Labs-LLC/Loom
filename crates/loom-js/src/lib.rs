//! Loom JS - JavaScript Engine
//!
//! Phase L13: JavaScript Engine
//! 
//! A JavaScript engine for the Loom browser built on Boa,
//! a pure Rust JavaScript engine.
//!
//! ## Features
//! - **Boa Integration**: Pure Rust JavaScript execution (no native dependencies)
//! - **DOM Bindings**: JavaScript can read and modify the DOM tree
//! - **Event Handling**: onclick, onsubmit, and other events fire JavaScript
//! - **Basic APIs**: document, window, console.log, fetch stub
//! - **Sandbox**: JavaScript runs isolated with no filesystem access
//!
//! ## Example
//! ```rust
//! use loom_js::JSEngine;
//!
//! let mut engine = JSEngine::new().unwrap();
//! 
//! // Execute JavaScript
//! let result = engine.execute("1 + 1");
//! assert!(result.success);
//! assert_eq!(result.value, "2");
//! 
//! // Use console API
//! engine.execute("console.log('Hello, World!')");
//! 
//! // Register event handler
//! engine.register_event_handler("btn1", "click", "alert('Clicked!')");
//! 
//! // Fire the event
//! engine.fire_event("btn1", "click");
//! ```

// Re-export main types
pub mod apis;
pub mod dom_bindings;
pub mod engine;
pub mod events;
pub mod sandbox;

// Re-export commonly used types
pub use engine::{JSEngine, JSEngineConfig, JSResult};
pub use events::{EventManager, Event, EventType, EventHandler};
pub use sandbox::{Sandbox, SandboxConfig, SecurityPolicy, SandboxError};
pub use dom_bindings::{DomBridge, DomChangeType};

/// Version of the JavaScript engine
pub const VERSION: &str = "0.2.0-L13";

/// Check if Boa JavaScript engine is available
pub fn is_engine_available() -> bool {
    // Boa is always available since it's a Rust crate
    true
}

/// Create a new JavaScript engine with default configuration
pub fn create_engine() -> anyhow::Result<JSEngine> {
    JSEngine::new()
}

/// Create a new JavaScript engine with strict sandboxing
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

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = create_engine();
        assert!(engine.is_ok());
    }

    #[test]
    fn test_sandboxed_engine() {
        let engine = create_sandboxed_engine();
        assert!(engine.is_ok());
    }

    #[test]
    fn test_console_hello_world() {
        let mut engine = JSEngine::new().unwrap();
        let result = engine.execute("console.log('Hello, World!')");
        assert!(result.success);
    }

    #[test]
    fn test_window_alert() {
        let mut engine = JSEngine::new().unwrap();
        let result = engine.execute("alert('Hello')");
        assert!(result.success);
    }

    #[test]
    fn test_document_getelementbyid() {
        let mut engine = JSEngine::new().unwrap();
        let result = engine.execute("document.getElementById('test')");
        assert!(result.success);
    }

    #[test]
    fn test_strict_mode_blocks_undeclared_vars() {
        let mut engine = JSEngine::new().unwrap();
        // In strict mode, assigning to undeclared variable should error
        let result = engine.execute("undeclaredVar = 1");
        assert!(!result.success);
    }

    #[test]
    fn test_event_handler_registration() {
        let mut engine = JSEngine::new().unwrap();
        
        engine.register_event_handler("myButton", "click", "console.log('clicked')");
        
        assert!(engine.has_event_handler("myButton", "click"));
        assert!(!engine.has_event_handler("myButton", "mouseover"));
    }

    #[test]
    fn test_click_handler_execution() {
        let mut engine = JSEngine::new().unwrap();
        
        // Set up a global variable
        engine.execute("var clickCount = 0");
        
        // Register a click handler that increments the counter
        engine.register_event_handler("btn1", "click", "clickCount++");
        
        // Fire the event
        engine.fire_event("btn1", "click");
        
        // Check the counter
        let count = engine.get_global("clickCount");
        assert_eq!(count, Some("1".to_string()));
    }

    #[test]
    fn test_button_onclick_example() {
        // This is the success criteria from the spec:
        // <button onclick="alert('hi')"> works
        let mut engine = JSEngine::new().unwrap();
        
        // Register the onclick handler (as would be parsed from HTML)
        engine.register_event_handler("btn1", "click", "alert('hi')");
        
        // Fire the click event
        let result = engine.fire_event("btn1", "click");
        
        // Should succeed (alert logs to console in our implementation)
        assert!(result.success);
    }

    #[test]
    fn test_fetch_stub() {
        let mut engine = JSEngine::new().unwrap();
        
        // fetch() should exist but return a rejected promise
        let result = engine.execute("fetch('https://example.com')");
        
        // In our stub, this actually succeeds but returns an error response object
        // The function call itself doesn't throw
        assert!(result.success);
    }

    #[test]
    fn test_global_variable_persistence() {
        let mut engine = JSEngine::new().unwrap();
        
        engine.execute("var x = 42");
        engine.execute("var y = x + 8");
        
        let y = engine.get_global("y");
        assert_eq!(y, Some("50".to_string()));
    }

    #[test]
    fn test_function_definition_and_call() {
        let mut engine = JSEngine::new().unwrap();
        
        engine.execute(r#"
            function greet(name) {
                return "Hello, " + name + "!";
            }
        "#);
        
        let result = engine.execute("greet('World')");
        assert!(result.success);
        assert_eq!(result.value, "Hello, World!");
    }

    #[test]
    fn test_eval_blocked_in_sandbox() {
        // Note: This test requires the sandbox to be applied
        // The actual blocking happens in Boa context setup
        let mut engine = JSEngine::new().unwrap();
        
        // eval should throw an error
        let result = engine.execute("eval('1 + 1')");
        
        // Our sandbox blocks eval
        assert!(!result.success);
        assert!(result.error.unwrap().contains("disabled"));
    }

    #[test]
    fn test_complex_script() {
        let mut engine = JSEngine::new().unwrap();
        
        let script = r#"
            var data = {
                items: [1, 2, 3, 4, 5],
                sum: function() {
                    var total = 0;
                    for (var i = 0; i < this.items.length; i++) {
                        total += this.items[i];
                    }
                    return total;
                }
            };
            
            data.sum();
        "#;
        
        let result = engine.execute(script);
        assert!(result.success);
        assert_eq!(result.value, "15");
    }
}
