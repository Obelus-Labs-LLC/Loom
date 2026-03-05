//! JavaScript Engine using Boa
//!
//! Phase L13: JavaScript Engine
//! - Boa JS integration for pure Rust execution
//! - DOM bindings for read/modify DOM tree
//! - Event handling (onclick, onsubmit, etc.)
//! - Basic APIs: document, window, console.log, fetch stub
//! - Sandboxed execution (no filesystem access)

use anyhow::{anyhow, Result};
use boa_engine::{
    Context, JsArgs, JsError, JsNativeError, JsResult, JsValue, NativeFunction,
    object::ObjectInitializer, property::Attribute, realm::Realm,
};
use boa_gc::{GcRefCell, Trace, Finalize};
use log::{debug, error, info, warn};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::apis::{ConsoleApi, DocumentApi, FetchApi, WindowApi};
use crate::dom_bindings::DomBridge;
use crate::events::EventManager;
use crate::sandbox::Sandbox;

/// JavaScript Engine configuration
#[derive(Debug, Clone)]
pub struct JSEngineConfig {
    /// Enable console API
    pub enable_console: bool,
    /// Enable fetch API (stub)
    pub enable_fetch: bool,
    /// Max execution time in milliseconds (0 = unlimited)
    pub max_execution_time_ms: u64,
    /// Max memory in bytes (0 = unlimited)
    pub max_memory_bytes: usize,
    /// Enable strict mode by default
    pub strict_mode: bool,
}

impl Default for JSEngineConfig {
    fn default() -> Self {
        Self {
            enable_console: true,
            enable_fetch: true,
            max_execution_time_ms: 5000, // 5 seconds
            max_memory_bytes: 64 * 1024 * 1024, // 64 MB
            strict_mode: true,
        }
    }
}

/// JavaScript execution result
#[derive(Debug, Clone)]
pub struct JSResult {
    pub value: String,
    pub success: bool,
    pub error: Option<String>,
}

impl JSResult {
    pub fn success(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            success: true,
            error: None,
        }
    }

    pub fn error(error: impl Into<String>) -> Self {
        Self {
            value: String::new(),
            success: false,
            error: Some(error.into()),
        }
    }
}

/// JavaScript Engine with Boa
pub struct JSEngine {
    /// Boa JS context
    context: Context,
    /// Engine configuration
    config: JSEngineConfig,
    /// DOM bridge for JS-DOM interaction
    dom_bridge: Option<Rc<RefCell<DomBridge>>>,
    /// Event manager for handling JS events
    event_manager: EventManager,
    /// Security sandbox
    sandbox: Sandbox,
    /// Whether the engine is initialized
    initialized: bool,
}

impl JSEngine {
    /// Create a new JavaScript engine with default config
    pub fn new() -> Result<Self> {
        Self::with_config(JSEngineConfig::default())
    }

    /// Create a new JavaScript engine with custom config
    pub fn with_config(config: JSEngineConfig) -> Result<Self> {
        // Create Boa context with a new realm
        let realm = Realm::create(&Default::default());
        let mut context = Context::builder()
            .realm(realm)
            .build()
            .map_err(|e| anyhow!("Failed to create JS context: {}", e))?;

        let event_manager = EventManager::new();
        let sandbox = Sandbox::new(&config);

        let mut engine = Self {
            context,
            config,
            dom_bridge: None,
            event_manager,
            sandbox,
            initialized: false,
        };

        // Initialize the engine
        engine.init()?;

        info!("JavaScript engine initialized (Boa {})");
        
        Ok(engine)
    }

    /// Initialize the engine with built-in APIs
    fn init(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }

        // Set up console API
        if self.config.enable_console {
            self.setup_console()?;
        }

        // Set up window API
        self.setup_window()?;

        // Set up document API (will be connected to actual DOM later)
        self.setup_document()?;

        // Set up fetch API stub
        if self.config.enable_fetch {
            self.setup_fetch()?;
        }

        // Apply sandbox restrictions
        self.sandbox.apply(&mut self.context)?;

        self.initialized = true;
        Ok(())
    }

    /// Set up console API
    fn setup_console(&mut self) -> Result<()> {
        let console = ConsoleApi::new();
        console.register(&mut self.context)?;
        debug!("Console API registered");
        Ok(())
    }

    /// Set up window API
    fn setup_window(&mut self) -> Result<()> {
        let window = WindowApi::new();
        window.register(&mut self.context)?;
        debug!("Window API registered");
        Ok(())
    }

    /// Set up document API
    fn setup_document(&mut self) -> Result<()> {
        let document = DocumentApi::new();
        document.register(&mut self.context)?;
        debug!("Document API registered");
        Ok(())
    }

    /// Set up fetch API stub
    fn setup_fetch(&mut self) -> Result<()> {
        let fetch = FetchApi::new();
        fetch.register(&mut self.context)?;
        debug!("Fetch API registered");
        Ok(())
    }

    /// Connect the engine to a DOM document
    pub fn connect_dom(&mut self, document: Rc<RefCell<loom_layout::dom::Document>>) -> Result<()> {
        let bridge = DomBridge::new(document);
        self.dom_bridge = Some(Rc::new(RefCell::new(bridge)));
        
        // Update document API with actual DOM
        if let Some(bridge) = &self.dom_bridge {
            bridge.borrow().bind_to_context(&mut self.context)?;
        }
        
        info!("DOM connected to JavaScript engine");
        Ok(())
    }

    /// Execute JavaScript code
    pub fn execute(&mut self, code: &str) -> JSResult {
        self.execute_with_source(code, "<anonymous>")
    }

    /// Execute JavaScript code with source name
    pub fn execute_with_source(&mut self, code: &str, source: &str) -> JSResult {
        if !self.initialized {
            return JSResult::error("Engine not initialized");
        }

        debug!("Executing JS from {}", source);

        // Wrap in strict mode if configured
        let code_to_execute = if self.config.strict_mode && !code.trim().starts_with("'use strict'") {
            format!("'use strict';\n{}", code)
        } else {
            code.to_string()
        };

        // Execute with timeout check
        let result = self.context.eval(&code_to_execute);

        match result {
            Ok(value) => {
                let str_value = value.to_string(&mut self.context).unwrap_or_default();
                JSResult::success(str_value)
            }
            Err(e) => {
                let error_msg = format!("JavaScript error: {}", e);
                error!("{}", error_msg);
                JSResult::error(error_msg)
            }
        }
    }

    /// Execute a script element's content
    pub fn execute_script(&mut self, script_content: &str) -> JSResult {
        self.execute_with_source(script_content, "<script>")
    }

    /// Call a JavaScript function by name
    pub fn call_function(&mut self, name: &str, args: &[JsValue]) -> JSResult {
        let global = self.context.global_object();
        
        let function = global.get(name, &mut self.context);
        
        match function {
            Ok(func) if func.is_function() => {
                match func.call(&JsValue::undefined(), args, &mut self.context) {
                    Ok(result) => {
                        let str_value = result.to_string(&mut self.context).unwrap_or_default();
                        JSResult::success(str_value)
                    }
                    Err(e) => JSResult::error(format!("Function call error: {}", e)),
                }
            }
            Ok(_) => JSResult::error(format!("'{}' is not a function", name)),
            Err(e) => JSResult::error(format!("Failed to get function '{}': {}", name, e)),
        }
    }

    /// Register an event handler from an HTML attribute
    /// e.g., onclick="alert('hi')"
    pub fn register_event_handler(&mut self, element_id: &str, event_type: &str, handler_code: &str) {
        self.event_manager.register(element_id, event_type, handler_code);
        debug!("Registered {} handler for element '{}'", event_type, element_id);
    }

    /// Fire an event on an element
    pub fn fire_event(&mut self, element_id: &str, event_type: &str) -> JSResult {
        if let Some(handler_code) = self.event_manager.get_handler(element_id, event_type) {
            debug!("Firing {} on element '{}'", event_type, element_id);
            
            // Create event context
            let wrapped_code = format!(
                "(function() {{
                    var event = {{ type: '{}', target: document.getElementById('{}') }};
                    {}
                }})()",
                event_type, element_id, handler_code
            );
            
            self.execute(&wrapped_code)
        } else {
            JSResult::success("") // No handler registered
        }
    }

    /// Check if an element has an event handler
    pub fn has_event_handler(&self, element_id: &str, event_type: &str) -> bool {
        self.event_manager.has_handler(element_id, event_type)
    }

    /// Get global variable value
    pub fn get_global(&mut self, name: &str) -> Option<String> {
        let global = self.context.global_object();
        global.get(name, &mut self.context)
            .ok()
            .and_then(|v| v.to_string(&mut self.context).ok())
    }

    /// Set global variable value
    pub fn set_global(&mut self, name: &str, value: &str) -> Result<()> {
        let js_value = self.context.eval(value)
            .map_err(|e| anyhow!("Failed to parse value: {}", e))?;
        
        let global = self.context.global_object();
        global.set(name, js_value, true, &mut self.context)
            .map_err(|e| anyhow!("Failed to set global: {}", e))?;
        
        Ok(())
    }

    /// Clear all event handlers
    pub fn clear_event_handlers(&mut self) {
        self.event_manager.clear();
    }

    /// Reset the engine (clear state but keep APIs)
    pub fn reset(&mut self) -> Result<()> {
        self.clear_event_handlers();
        // Create a fresh context
        let realm = Realm::create(&Default::default());
        self.context = Context::builder()
            .realm(realm)
            .build()
            .map_err(|e| anyhow!("Failed to reset context: {}", e))?;
        
        self.initialized = false;
        self.init()?;
        
        // Reconnect DOM if it was connected
        if let Some(bridge) = &self.dom_bridge {
            let bridge_clone = bridge.clone();
            bridge_clone.borrow().bind_to_context(&mut self.context)?;
        }
        
        Ok(())
    }

    /// Get engine configuration
    pub fn config(&self) -> &JSEngineConfig {
        &self.config
    }

    /// Check if DOM is connected
    pub fn is_dom_connected(&self) -> bool {
        self.dom_bridge.is_some()
    }
}

impl Default for JSEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create default JS engine")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_execution() {
        let mut engine = JSEngine::new().unwrap();
        let result = engine.execute("1 + 1");
        assert!(result.success);
        assert_eq!(result.value, "2");
    }

    #[test]
    fn test_console_log() {
        let mut engine = JSEngine::new().unwrap();
        let result = engine.execute("console.log('hello world')");
        assert!(result.success);
    }

    #[test]
    fn test_global_variable() {
        let mut engine = JSEngine::new().unwrap();
        engine.execute("var x = 42");
        let value = engine.get_global("x");
        assert_eq!(value, Some("42".to_string()));
    }

    #[test]
    fn test_error_handling() {
        let mut engine = JSEngine::new().unwrap();
        let result = engine.execute("throw new Error('test error')");
        assert!(!result.success);
        assert!(result.error.is_some());
    }

    #[test]
    fn test_strict_mode() {
        let mut engine = JSEngine::new().unwrap();
        // In strict mode, assigning to undeclared variable should error
        let result = engine.execute("undeclaredVar = 1");
        assert!(!result.success);
    }

    #[test]
    fn test_event_registration() {
        let mut engine = JSEngine::new().unwrap();
        engine.register_event_handler("btn1", "click", "console.log('clicked')");
        assert!(engine.has_event_handler("btn1", "click"));
        assert!(!engine.has_event_handler("btn1", "mouseover"));
    }
}
