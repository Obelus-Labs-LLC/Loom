//! FabricOS V8 JavaScript Engine
//!
//! JsEngine implementation using FabricOS V8 FFI bindings.
//! Ready for integration once libv8_shim.a is available.
//!
//! STATUS: Skeleton implementation - BLOCKED until FabricOS L13.7

use super::ffi::*;
use crate::engine_trait::{
    HeapStatistics, JsEngine, JsEngineError, JsException, JsMetrics, JsObject, JsResult, JsValue,
    MemoryPressure,
};
use std::ffi::{CStr, CString};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Instant;

/// FabricOS V8 JavaScript Engine
///
/// This engine uses the FabricOS V8 shim via FFI.
/// It requires libv8_shim.a to be linked at build time.
pub struct FabricOSV8Engine {
    /// V8 isolate handle
    isolate: V8IsolateHandle,
    /// Current context handle
    context: V8ContextHandle,
    /// Whether initialized
    initialized: AtomicBool,
    /// Performance metrics
    metrics: Mutex<JsMetrics>,
    /// Memory limit in bytes
    memory_limit: usize,
    /// Console log callback
    log_callback: Option<fn(&str)>,
}

impl FabricOSV8Engine {
    /// Create new FabricOS V8 engine
    ///
    /// # Panics
    /// Panics if V8 platform is not initialized
    pub fn new() -> Result<Self, JsEngineError> {
        // BLOCKED: This will work once libv8_shim.a is available
        // For now, return a stub error
        #[cfg(not(feature = "fabricos-v8"))]
        {
            return Err(JsEngineError::InitializationFailed(
                "FabricOS V8 not available - libv8_shim.a not linked (L13.7 pending)".to_string()
            ));
        }

        #[cfg(feature = "fabricos-v8")]
        {
            let config = V8Config {
                heap_size_mb: 128,
                enable_jit: 0, // JIT-less mode for bare metal
                use_snapshot: 1,
                snapshot_path: std::ptr::null(),
            };

            let isolate = unsafe { fabricos_v8_isolate_create(&config) };
            if isolate.is_null() {
                return Err(JsEngineError::InitializationFailed(
                    "Failed to create V8 isolate".to_string()
                ));
            }

            let context = unsafe { fabricos_v8_context_create(isolate) };
            if context.is_null() {
                unsafe { fabricos_v8_isolate_dispose(isolate) };
                return Err(JsEngineError::InitializationFailed(
                    "Failed to create V8 context".to_string()
                ));
            }

            Ok(Self {
                isolate,
                context,
                initialized: AtomicBool::new(false),
                metrics: Mutex::new(JsMetrics::default()),
                memory_limit: 128 * 1024 * 1024,
                log_callback: None,
            })
        }
    }

    /// Check if FabricOS V8 is available
    pub fn is_available() -> bool {
        cfg!(feature = "fabricos-v8")
    }

    /// Initialize console API
    fn init_console(&self) {
        extern "C" fn log_handler(msg: *const std::os::raw::c_char) {
            if !msg.is_null() {
                let msg = unsafe { CStr::from_ptr(msg) };
                eprintln!("[JS] {}", msg.to_string_lossy());
            }
        }

        extern "C" fn error_handler(msg: *const std::os::raw::c_char) {
            if !msg.is_null() {
                let msg = unsafe { CStr::from_ptr(msg) };
                eprintln!("[JS ERROR] {}", msg.to_string_lossy());
            }
        }

        #[cfg(feature = "fabricos-v8")]
        unsafe {
            fabricos_v8_console_install(
                self.context,
                Some(log_handler),
                Some(error_handler),
            );
        }
    }

    /// Convert V8 value handle to JsValue
    unsafe fn v8_to_js(&self, value: V8ValueHandle) -> JsValue {
        if value.is_null() {
            return JsValue::Null;
        }

        #[cfg(feature = "fabricos-v8")]
        {
            let typ = fabricos_v8_value_type(value);
            
            match typ {
                value_types::UNDEFINED => JsValue::Undefined,
                value_types::NULL => JsValue::Null,
                value_types::BOOL => {
                    JsValue::Bool(fabricos_v8_value_to_bool(value) != 0)
                }
                value_types::NUMBER => {
                    JsValue::Number(fabricos_v8_value_to_number(value))
                }
                value_types::STRING => {
                    let s = fabricos_v8_value_to_string(value);
                    let result = if s.is_null() {
                        JsValue::String(String::new())
                    } else {
                        let cstr = CStr::from_ptr(s);
                        let string = cstr.to_string_lossy().into_owned();
                        fabricos_v8_string_free(s);
                        JsValue::String(string)
                    };
                    fabricos_v8_value_dispose(value);
                    result
                }
                value_types::OBJECT => {
                    // For now, just return a placeholder object
                    // Full implementation would traverse properties
                    fabricos_v8_value_dispose(value);
                    JsValue::Object(JsObject { id: 0, prototype: None })
                }
                _ => JsValue::Undefined,
            }
        }
        
        #[cfg(not(feature = "fabricos-v8"))]
        {
            JsValue::Undefined
        }
    }

    /// Convert JsValue to V8 value
    unsafe fn js_to_v8(&self, value: &JsValue) -> V8ValueHandle {
        #[cfg(feature = "fabricos-v8")]
        match value {
            JsValue::Undefined => fabricos_v8_undefined_new(self.isolate),
            JsValue::Null => fabricos_v8_null_new(self.isolate),
            JsValue::Bool(b) => fabricos_v8_bool_new(self.isolate, *b as i32),
            JsValue::Number(n) => fabricos_v8_number_new(self.isolate, *n),
            JsValue::String(s) => {
                let cstr = CString::new(s.as_str()).unwrap_or_default();
                fabricos_v8_string_new(self.isolate, cstr.as_ptr(), s.len())
            }
            _ => fabricos_v8_undefined_new(self.isolate),
        }
        
        #[cfg(not(feature = "fabricos-v8"))]
        {
            std::ptr::null_mut()
        }
    }
}

impl JsEngine for FabricOSV8Engine {
    fn initialize(&mut self) -> Result<(), JsEngineError> {
        if self.initialized.load(Ordering::SeqCst) {
            return Ok(());
        }

        #[cfg(feature = "fabricos-v8")]
        unsafe {
            fabricos_v8_context_enter(self.context);
            self.init_console();
            fabricos_v8_context_exit(self.context);
        }

        self.initialized.store(true, Ordering::SeqCst);
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), JsEngineError> {
        if !self.initialized.load(Ordering::SeqCst) {
            return Ok(());
        }

        #[cfg(feature = "fabricos-v8")]
        unsafe {
            fabricos_v8_context_dispose(self.context);
            fabricos_v8_isolate_dispose(self.isolate);
        }

        self.initialized.store(false, Ordering::SeqCst);
        Ok(())
    }

    fn eval(&mut self, script: &str) -> JsResult {
        let start = Instant::now();

        #[cfg(feature = "fabricos-v8")]
        {
            unsafe {
                fabricos_v8_context_enter(self.context);

                let source = CString::new(script).map_err(|_| {
                    JsEngineError::ScriptParseError("Script contains null byte".to_string())
                })?;
                
                let filename = CString::new("<eval>").unwrap();
                
                let result = fabricos_v8_script_run(
                    self.context,
                    source.as_ptr(),
                    script.len(),
                    filename.as_ptr(),
                );

                fabricos_v8_context_exit(self.context);

                // Update metrics
                {
                    let mut metrics = self.metrics.lock().unwrap();
                    metrics.scripts_executed += 1;
                    metrics.total_execution_time_ms += start.elapsed().as_millis() as u64;
                }

                if result.success != 0 {
                    let value = self.v8_to_js(result.value);
                    Ok(crate::engine_trait::JsExecutionResult {
                        value,
                        execution_time_ms: start.elapsed().as_millis() as u64,
                    })
                } else {
                    let error = if result.error.is_null() {
                        "Unknown error".to_string()
                    } else {
                        CStr::from_ptr(result.error)
                            .to_string_lossy()
                            .into_owned()
                    };
                    
                    Err(JsEngineError::ScriptExecutionError(error))
                }
            }
        }
        
        #[cfg(not(feature = "fabricos-v8"))]
        {
            // Stub implementation for when V8 is not available
            Err(JsEngineError::ScriptExecutionError(
                "FabricOS V8 not available - libv8_shim.a not linked (L13.7 pending)".to_string()
            ))
        }
    }

    fn eval_with_timeout(&mut self, script: &str, _timeout_ms: u64) -> JsResult {
        // For now, ignore timeout - full implementation would use V8's
        // script timeout or interrupt mechanism
        self.eval(script)
    }

    fn call_function(&mut self, _name: &str, _args: &[JsValue]) -> JsResult {
        // BLOCKED: Implement after basic eval works
        Err(JsEngineError::NotImplemented(
            "Function calls not yet implemented".to_string()
        ))
    }

    fn set_global(&mut self, _name: &str, _value: JsValue) -> Result<(), JsEngineError> {
        // BLOCKED: Implement after basic eval works
        Err(JsEngineError::NotImplemented(
            "Global variables not yet implemented".to_string()
        ))
    }

    fn get_global(&mut self, _name: &str) -> Result<JsValue, JsEngineError> {
        // BLOCKED: Implement after basic eval works
        Err(JsEngineError::NotImplemented(
            "Global variables not yet implemented".to_string()
        ))
    }

    fn create_object(&mut self) -> Result<JsObject, JsEngineError> {
        // BLOCKED: Implement after basic eval works
        Err(JsEngineError::NotImplemented(
            "Object creation not yet implemented".to_string()
        ))
    }

    fn set_property(&mut self, _obj: &JsObject, _name: &str, _value: JsValue) -> Result<(), JsEngineError> {
        // BLOCKED: Implement after basic eval works
        Err(JsEngineError::NotImplemented(
            "Property setting not yet implemented".to_string()
        ))
    }

    fn get_property(&mut self, _obj: &JsObject, _name: &str) -> Result<JsValue, JsEngineError> {
        // BLOCKED: Implement after basic eval works
        Err(JsEngineError::NotImplemented(
            "Property getting not yet implemented".to_string()
        ))
    }

    fn memory_pressure(&mut self, _pressure: MemoryPressure) {
        #[cfg(feature = "fabricos-v8")]
        unsafe {
            match _pressure {
                MemoryPressure::Critical | MemoryPressure::Moderate => {
                    fabricos_v8_isolate_request_gc(self.isolate);
                }
                _ => {}
            }
        }
    }

    fn heap_statistics(&self) -> HeapStatistics {
        #[cfg(feature = "fabricos-v8")]
        unsafe {
            let mut total: usize = 0;
            let mut used: usize = 0;
            fabricos_v8_isolate_heap_stats(self.isolate, &mut total, &mut used);
            
            HeapStatistics {
                total_heap_size: total,
                used_heap_size: used,
                heap_size_limit: self.memory_limit,
                total_physical_size: total,
                total_available_size: total - used,
                ..Default::default()
            }
        }
        
        #[cfg(not(feature = "fabricos-v8"))]
        {
            HeapStatistics::default()
        }
    }

    fn metrics(&self) -> JsMetrics {
        self.metrics.lock().unwrap().clone()
    }

    fn reset_metrics(&mut self) {
        *self.metrics.lock().unwrap() = JsMetrics::default();
    }

    fn set_memory_limit(&mut self, limit_bytes: usize) {
        self.memory_limit = limit_bytes;
        #[cfg(feature = "fabricos-v8")]
        unsafe {
            fabricos_v8_isolate_set_memory_limit(self.isolate, (limit_bytes / 1024 / 1024) as i32);
        }
    }

    fn set_stack_size(&mut self, _size_bytes: usize) {
        // V8 stack size is typically set at isolate creation
        // This would require recreating the isolate
    }
}

impl Drop for FabricOSV8Engine {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

impl Default for FabricOSV8Engine {
    fn default() -> Self {
        Self::new().expect("Failed to create FabricOS V8 engine")
    }
}

/// Initialize V8 platform (call once at startup)
///
/// # Safety
/// Must be called before any FabricOSV8Engine instances are created
pub unsafe fn init_v8_platform() -> Result<(), JsEngineError> {
    #[cfg(feature = "fabricos-v8")]
    {
        let config = V8Config {
            heap_size_mb: 0, // Use default
            enable_jit: 0,
            use_snapshot: 1,
            snapshot_path: std::ptr::null(),
        };

        if fabricos_v8_init(&config) != 0 {
            return Err(JsEngineError::InitializationFailed(
                "V8 platform initialization failed".to_string()
            ));
        }
    }
    
    Ok(())
}

/// Shutdown V8 platform (call once at shutdown)
pub fn shutdown_v8_platform() {
    #[cfg(feature = "fabricos-v8")]
    unsafe {
        fabricos_v8_shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fabricos_v8_not_available() {
        // When fabricos-v8 feature is not enabled, engine creation should fail gracefully
        let result = FabricOSV8Engine::new();
        
        #[cfg(not(feature = "fabricos-v8"))]
        assert!(result.is_err());
        
        #[cfg(feature = "fabricos-v8")]
        assert!(result.is_ok());
    }

    // Note: Full integration tests require libv8_shim.a
    // These will be enabled when FabricOS L13.7 completes
}
