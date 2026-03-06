# L23: FabricOS V8 Integration

## Overview

This document describes the FabricOS V8 JavaScript engine integration for Loom.

## Status

| Component | Status | Notes |
|:---|:---|:---|
| FFI Bindings | ✅ Complete | `fabricos_v8/ffi.rs` - All C functions declared |
| Engine Implementation | ✅ Skeleton | `fabricos_v8/engine.rs` - Ready for linking |
| Test Harness | ✅ Complete | `tests/fabricos_v8_integration.rs` - 1+2=3 validation |
| Linking | ⏳ **BLOCKED** | Waiting for `libv8_shim.a` (FabricOS L13.7) |

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                   Loom Browser (JS Engine)                   │
├─────────────────────────────────────────────────────────────┤
│  JsEngine Trait                                              │
│  ├─ BoaJsEngine (AI-Native mode)                            │
│  ├─ V8Engine (Traditional, using rust-v8 crate)             │
│  └─ FabricOSV8Engine (Traditional, bare metal) ⬅ NEW        │
├─────────────────────────────────────────────────────────────┤
│  FFI Layer (fabricos_v8/ffi.rs)                              │
│  ├─ fabricos_v8_init/shutdown                               │
│  ├─ fabricos_v8_isolate_create/dispose                      │
│  ├─ fabricos_v8_context_create/enter/exit                   │
│  ├─ fabricos_v8_script_run                                  │
│  └─ fabricos_v8_value_* operations                          │
├─────────────────────────────────────────────────────────────┤
│  C++ V8 Shim (FabricOS kernel/src/v8_shim/)                  │
│  ├─ v8_fabricos_shim.cc                                     │
│  └─ libv8_shim.a ⬅ BLOCKED until L13.7                     │
├─────────────────────────────────────────────────────────────┤
│  V8 Engine (FabricOS vendor/v8/)                            │
│  └─ libv8.a, snapshot_blob.bin                              │
└─────────────────────────────────────────────────────────────┘
```

## Files

| File | Description |
|:---|:---|
| `src/fabricos_v8/mod.rs` | Module exports, validation harness |
| `src/fabricos_v8/ffi.rs` | FFI declarations for C++ shim |
| `src/fabricos_v8/engine.rs` | JsEngine trait implementation |
| `tests/fabricos_v8_integration.rs` | Integration tests (1+2=3) |

## FFI Functions

### Platform Lifecycle
```rust
fabricos_v8_init(config: *const V8Config) -> c_int;
fabricos_v8_shutdown();
fabricos_v8_version() -> *const c_char;
```

### Isolate Management
```rust
fabricos_v8_isolate_create(config: *const V8Config) -> V8IsolateHandle;
fabricos_v8_isolate_dispose(isolate: V8IsolateHandle);
fabricos_v8_isolate_heap_stats(isolate, total, used);
```

### Script Execution
```rust
fabricos_v8_script_run(
    context: V8ContextHandle,
    source: *const c_char,
    source_len: usize,
    filename: *const c_char,
) -> V8ScriptResult;
```

## Integration Test: 1+2=3

```rust
use loom_js::fabricos_v8::{FabricOSV8Engine, init_v8_platform, shutdown_v8_platform};
use loom_js::engine_trait::JsEngine;

// Initialize platform
unsafe { init_v8_platform().unwrap(); }

// Create and initialize engine
let mut engine = FabricOSV8Engine::new().unwrap();
engine.initialize().unwrap();

// Execute 1+2
let result = engine.eval("1 + 2").unwrap();
assert_eq!(result.value, JsValue::Number(3.0));

// Cleanup
engine.shutdown().unwrap();
shutdown_v8_platform();
```

## Running Tests

### Without libv8_shim.a (compilation check)
```bash
cargo test --package loom-js --test fabricos_v8_integration
```

### With libv8_shim.a (full validation)
```bash
# 1. Copy libv8_shim.a to link path
cp /path/to/libv8_shim.a /path/to/loom/lib/

# 2. Enable feature and run tests
cargo test --package loom-js --test fabricos_v8_integration --features fabricos-v8

# 3. Run 1+2=3 validation
cargo test --package loom-js --test fabricos_v8_integration --features fabricos-v8 -- --ignored
```

## Blocked Until

FabricOS **L13.7** completes:
- `libv8_shim.a` built and available
- `snapshot_blob.bin` generated
- FFI bindings tested

## Next Steps

1. Wait for FabricOS L13.7 completion
2. Copy `libv8_shim.a` to link path
3. Enable `fabricos-v8` feature in Cargo.toml
4. Run `test_one_plus_two_equals_three`
5. Verify all integration tests pass
6. Enable in production builds

## Notes

- Feature flag `fabricos-v8` is currently disabled
- Engine returns graceful error when library unavailable
- Full implementation is ready, just needs linking
- Test harness validates 1+2=3, console.log, error handling
