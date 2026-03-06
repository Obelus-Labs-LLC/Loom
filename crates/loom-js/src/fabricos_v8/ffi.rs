//! FabricOS V8 FFI Bindings
//!
//! Rust FFI declarations for the FabricOS V8 C++ shim.
//! These functions are implemented in kernel/src/v8_shim/v8_fabricos_shim.cc
//! and linked via libv8_shim.a.
//!
//! BLOCKED UNTIL: FabricOS L13.7 completes (libv8_shim.a ready)

use std::os::raw::{c_char, c_double, c_int, c_void};

/// Opaque handle to V8 isolate
pub type V8IsolateHandle = *mut c_void;

/// Opaque handle to V8 context
pub type V8ContextHandle = *mut c_void;

/// Opaque handle to V8 value
pub type V8ValueHandle = *mut c_void;

/// V8 script execution result
#[repr(C)]
pub struct V8ScriptResult {
    /// Success flag
    pub success: c_int,
    /// Result value (if success)
    pub value: V8ValueHandle,
    /// Error message (if failed)
    pub error: *const c_char,
    /// Line number of error
    pub error_line: c_int,
    /// Column number of error
    pub error_column: c_int,
}

/// V8 initialization configuration
#[repr(C)]
pub struct V8Config {
    /// Heap size limit in MB
    pub heap_size_mb: c_int,
    /// Enable JIT compilation
    pub enable_jit: c_int,
    /// Enable snapshots
    pub use_snapshot: c_int,
    /// Snapshot blob path (or null for embedded)
    pub snapshot_path: *const c_char,
}

extern "C" {
    // =========================================================================
    // V8 Platform Lifecycle
    // =========================================================================

    /// Initialize V8 platform (must be called once)
    ///
    /// # Safety
    /// Must be called before any other V8 functions
    pub fn fabricos_v8_init(config: *const V8Config) -> c_int;

    /// Shutdown V8 platform
    ///
    /// # Safety
    /// Must be called after all isolates are disposed
    pub fn fabricos_v8_shutdown();

    /// Get V8 version string
    pub fn fabricos_v8_version() -> *const c_char;

    // =========================================================================
    // Isolate Management
    // =========================================================================

    /// Create new V8 isolate
    pub fn fabricos_v8_isolate_create(config: *const V8Config) -> V8IsolateHandle;

    /// Dispose of V8 isolate
    ///
    /// # Safety
    /// Handle must be valid and not already disposed
    pub fn fabricos_v8_isolate_dispose(isolate: V8IsolateHandle);

    /// Set memory limit for isolate
    pub fn fabricos_v8_isolate_set_memory_limit(isolate: V8IsolateHandle, limit_mb: c_int);

    /// Get heap statistics
    pub fn fabricos_v8_isolate_heap_stats(
        isolate: V8IsolateHandle,
        total_size: *mut usize,
        used_size: *mut usize,
    );

    /// Request garbage collection
    pub fn fabricos_v8_isolate_request_gc(isolate: V8IsolateHandle);

    // =========================================================================
    // Context Management
    // =========================================================================

    /// Create new context in isolate
    pub fn fabricos_v8_context_create(isolate: V8IsolateHandle) -> V8ContextHandle;

    /// Dispose of context
    ///
    /// # Safety
    /// Handle must be valid
    pub fn fabricos_v8_context_dispose(context: V8ContextHandle);

    /// Enter context scope
    ///
    /// # Safety
    /// Must call exit_context before disposing
    pub fn fabricos_v8_context_enter(context: V8ContextHandle);

    /// Exit context scope
    pub fn fabricos_v8_context_exit(context: V8ContextHandle);

    /// Get global object
    pub fn fabricos_v8_context_global(context: V8ContextHandle) -> V8ValueHandle;

    // =========================================================================
    // Script Execution
    // =========================================================================

    /// Execute JavaScript script
    ///
    /// # Safety
    /// Context must be entered
    pub fn fabricos_v8_script_run(
        context: V8ContextHandle,
        source: *const c_char,
        source_len: usize,
        filename: *const c_char,
    ) -> V8ScriptResult;

    /// Compile script (for repeated execution)
    pub fn fabricos_v8_script_compile(
        context: V8ContextHandle,
        source: *const c_char,
        source_len: usize,
        filename: *const c_char,
    ) -> *mut c_void; // Script handle

    /// Run compiled script
    pub fn fabricos_v8_compiled_script_run(
        context: V8ContextHandle,
        script: *mut c_void,
    ) -> V8ScriptResult;

    /// Dispose compiled script
    pub fn fabricos_v8_compiled_script_dispose(script: *mut c_void);

    // =========================================================================
    // Value Operations
    // =========================================================================

    /// Dispose of value handle
    pub fn fabricos_v8_value_dispose(value: V8ValueHandle);

    /// Get value type
    pub fn fabricos_v8_value_type(value: V8ValueHandle) -> c_int;
    // Types: 0=undefined, 1=null, 2=bool, 3=number, 4=string, 5=object, 6=function

    /// Convert value to boolean
    pub fn fabricos_v8_value_to_bool(value: V8ValueHandle) -> c_int;

    /// Convert value to number
    pub fn fabricos_v8_value_to_number(value: V8ValueHandle) -> c_double;

    /// Convert value to string
    /// Caller must free returned string with fabricos_v8_string_free
    pub fn fabricos_v8_value_to_string(value: V8ValueHandle) -> *mut c_char;

    /// Free string returned by V8
    pub fn fabricos_v8_string_free(s: *mut c_char);

    /// Create number value
    pub fn fabricos_v8_number_new(isolate: V8IsolateHandle, value: c_double) -> V8ValueHandle;

    /// Create string value
    pub fn fabricos_v8_string_new(
        isolate: V8IsolateHandle,
        s: *const c_char,
        len: usize,
    ) -> V8ValueHandle;

    /// Create boolean value
    pub fn fabricos_v8_bool_new(isolate: V8IsolateHandle, value: c_int) -> V8ValueHandle;

    /// Create undefined value
    pub fn fabricos_v8_undefined_new(isolate: V8IsolateHandle) -> V8ValueHandle;

    /// Create null value
    pub fn fabricos_v8_null_new(isolate: V8IsolateHandle) -> V8ValueHandle;

    // =========================================================================
    // Object Operations
    // =========================================================================

    /// Create empty object
    pub fn fabricos_v8_object_new(isolate: V8IsolateHandle) -> V8ValueHandle;

    /// Set property on object
    pub fn fabricos_v8_object_set(
        object: V8ValueHandle,
        key: *const c_char,
        value: V8ValueHandle,
    ) -> c_int;

    /// Get property from object
    pub fn fabricos_v8_object_get(
        object: V8ValueHandle,
        key: *const c_char,
    ) -> V8ValueHandle;

    // =========================================================================
    // Console API
    // =========================================================================

    /// Install console object with callbacks
    pub fn fabricos_v8_console_install(
        context: V8ContextHandle,
        log_callback: Option<extern "C" fn(*const c_char)>,
        error_callback: Option<extern "C" fn(*const c_char)>,
    );

    // =========================================================================
    // Exception Handling
    // =========================================================================

    /// Check if exception is pending
    pub fn fabricos_v8_exception_pending(context: V8ContextHandle) -> c_int;

    /// Get and clear pending exception
    pub fn fabricos_v8_exception_get(context: V8ContextHandle) -> V8ValueHandle;

    /// Clear pending exception
    pub fn fabricos_v8_exception_clear(context: V8ContextHandle);

    /// Get exception message
    pub fn fabricos_v8_exception_message(exc: V8ValueHandle) -> *mut c_char;

    /// Get exception stack trace
    pub fn fabricos_v8_exception_stack_trace(exc: V8ValueHandle) -> *mut c_char;
}

/// V8 value type constants
pub mod value_types {
    pub const UNDEFINED: i32 = 0;
    pub const NULL: i32 = 1;
    pub const BOOL: i32 = 2;
    pub const NUMBER: i32 = 3;
    pub const STRING: i32 = 4;
    pub const OBJECT: i32 = 5;
    pub const FUNCTION: i32 = 6;
}

/// Convert V8ScriptResult to Rust result
///
/// # Safety
/// Must be called immediately after script execution while handles are valid
pub unsafe fn result_to_rust(result: V8ScriptResult) -> Result<*mut c_void, String> {
    if result.success != 0 {
        Ok(result.value)
    } else {
        let error = if result.error.is_null() {
            "Unknown error".to_string()
        } else {
            std::ffi::CStr::from_ptr(result.error)
                .to_string_lossy()
                .into_owned()
        };
        Err(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_types() {
        assert_eq!(value_types::UNDEFINED, 0);
        assert_eq!(value_types::NUMBER, 3);
        assert_eq!(value_types::STRING, 4);
    }

    // Note: FFI function tests require libv8_shim.a to be linked
    // These are integration tests that will be enabled when L13.7 completes
}
