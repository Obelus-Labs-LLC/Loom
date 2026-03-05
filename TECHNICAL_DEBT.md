# Loom Technical Debt Register

## Overview

This document tracks known technical debt, temporary exclusions, and deferred work in the Loom browser codebase. Items are prioritized by risk and target resolution phase.

## Debt Register

| ID | Item | Phase | Target | Risk | Status |
|:---|:---|:---|:---|:---|:---|
| TD-017 | loom-js Boa API drift | L13 | L23 V8 replacement | Low | **Excluded from build** - Boa engine has API incompatibilities. V8 will replace Boa in L23. |
| TD-018 | loom-webgl wgpu API changes | L17 | L29+ | Low | **FIXED** - wgpu 0.19 API updated, `get_compilation_info()` removed, `module()` getter added. |
| TD-019 | loom-media dependency updates | L14 | Maintenance | Low | **FIXED** - Dependencies verified compatible (png 0.17, jpeg-decoder 0.3, webp 0.3). |
| TD-020 | loom-security no_std HashMap | L15 | Post-L23 | Low | **FIXED** - Replaced with `hashbrown::HashMap`, added `ahash` feature. |
| TD-021 | loom-content type annotations | L6 | Post-L23 | Low | **FIXED** - Type annotations added, alloc imports removed. |
| TD-022 | loom-ai test imports | L18 | L29 | Low | Test-only: `BrowserMode` import path issues in tests. |
| TD-023 | no_std feature unification | L24-L28 | L29 | Medium | **RESOLVED** - L24-L28 crates (WASM, Service Workers, WebRTC) restored to workspace. |

## Excluded Crates (Temporary)

Only one crate remains excluded from the workspace build:

```toml
# In root Cargo.toml workspace.members:
"crates/loom-js"         # TD-017 - Boa API drift, V8 replacement planned
```

### Resolved Crates (Restored)
- ✅ `crates/loom-webgl` - TD-018 fixed (wgpu 0.19 API)
- ✅ `crates/loom-media` - TD-019 fixed (dependencies verified)
- ✅ `crates/loom-security` - TD-020 fixed (hashbrown HashMap)
- ✅ `crates/loom-content` - TD-021 fixed (type annotations)
- ✅ `crates/loom-wasm` - L24 restored (no_std compatible)
- ✅ `crates/loom-serviceworker` - L25 restored (no_std compatible)
- ✅ `crates/loom-webrtc` - L27 restored (no_std compatible)

### Resolution Plan

1. **✅ COMPLETED**: TD-018, TD-019, TD-020, TD-021, L24, L25, L27 - All restored
2. **L23 (V8 Integration)**: Replace Boa with V8 engine (TD-017)
3. **L29+ (Performance)**: 60fps benchmark, React Fiber work loop
4. **L30+ (New Features)**: Multi-user sessions, PWA support, Extension API

## Build Configuration

### Current Working Configuration
- Target: `x86_64-pc-windows-msvc` (desktop)
- Active crates: 12/13 (all except loom-js)
  - Core: loom-core, loom-layout, loom-chrome, loom-render, loom-content
  - Media: loom-media, loom-webgl
  - Security: loom-security
  - Advanced: loom-wasm, loom-serviceworker, loom-webrtc
  - AI/Tools: loom-ai, loom-devtools, loom-extensions, loom-a11y
- Excluded: loom-js (TD-017, V8 replacement planned)

### FabricOS Build
- Target: `x86_64-unknown-none` (bare metal)
- Requires: Uncomment `.cargo/config.toml` FabricOS section
- Note: L24-L28 no_std crates (WASM, Service Workers, WebRTC) designed for FabricOS

---

*Last updated: March 3, 2026*
