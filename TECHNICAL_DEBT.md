# Loom Technical Debt Register

## Overview

This document tracks known technical debt, temporary exclusions, and deferred work in the Loom browser codebase. Items are prioritized by risk and target resolution phase.

## Debt Register

| ID | Item | Phase | Target | Risk | Status |
|:---|:---|:---|:---|:---|:---|
| TD-017 | loom-js Boa API drift | L13 | L23 V8 replacement | Low | **Excluded from build** - Boa engine has API incompatibilities with latest version. V8 will replace Boa in L23 anyway. |
| TD-018 | loom-webgl wgpu API changes | L17 | L29+ | Low | **Excluded from build** - wgpu 0.20 API drift, needs update for latest wgpu. |
| TD-019 | loom-media dependency updates | L14 | Maintenance | Low | **Excluded from build** - Image crate API changes, needs dependency refresh. |
| TD-020 | loom-security no_std HashMap | L15 | Post-L23 | Low | **FIXED** - Replaced with `hashbrown::HashMap`, added `ahash` feature. |
| TD-021 | loom-content type annotations | L6 | Post-L23 | Low | **FIXED** - Type annotations added, alloc imports removed. Tests have pre-existing issues. |
| TD-022 | loom-ai test imports | L18 | L29 | Low | Test-only: `BrowserMode` import path issues in tests. |
| TD-023 | no_std feature unification | L24-L28 | L29 | Medium | L24-L28 crates (WASM, Service Workers, WebRTC) need consistent no_std strategy across workspace. |

## Excluded Crates (Temporary)

The following crates are temporarily excluded from the workspace build to enable L29 Performance work:

```toml
# In root Cargo.toml workspace.members:
# "crates/loom-js"         # TD-017
# "crates/loom-webgl"      # TD-018  
# "crates/loom-media"      # TD-019
# "crates/loom-security"   # TD-020
# "crates/loom-content"    # TD-021
# "crates/loom-wasm"       # TD-023
# "crates/loom-serviceworker" # TD-023
# "crates/loom-webrtc"     # TD-023
```

### Resolution Plan

1. **Post-L23 (V8 Integration)**: Repair loom-js, loom-security, loom-content
2. **Post-L26 (Media Codecs)**: Repair loom-media  
3. **Post-L29 (Performance)**: Repair loom-webgl
4. **L30+ (Maintenance)**: Unified no_std strategy for L24-L28

## Build Configuration

### Current Working Configuration
- Target: `x86_64-pc-windows-msvc` (desktop)
- Active crates: loom-core, loom-layout, loom-chrome, loom-render, loom-ai, loom-devtools, loom-extensions, loom-a11y
- Tests passing: 90+

### FabricOS Build
- Target: `x86_64-unknown-none` (bare metal)
- Requires: Uncomment `.cargo/config.toml` FabricOS section
- Note: L24-L28 no_std crates (WASM, Service Workers, WebRTC) designed for FabricOS

---

*Last updated: March 2026*
