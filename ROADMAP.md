# LOOM BROWSER — COMPLETE PHASE ROADMAP

## Overview

AI-Native web browser for FabricOS. Dual-mode architecture: Traditional (full web compatibility) and AI-Native (intent-driven minimal chrome).

**Repository:** https://github.com/Obelus-Labs-LLC/Loom
**License:** GPL-3.0
**Target:** FabricOS microkernel + Desktop (Windows/Linux)

---

## Phases

| Phase | Name | Deliverables | Success Criteria | Dependencies |
|:---|:---|:---|:---|:---|
| L0 | Bootstrap | Project structure, wgpu init, window creation | Window opens, 60fps render loop | None |
| L1 | wgpu Surface | Desktop rendering backend, swapchain | Smooth 60fps on Windows/Linux | L0 |
| L2 | Design System | Chromatic temperature, tension curves, typography | Colors adapt to time of day, shapes render | L1 |
| L3 | HTML/CSS Parsing | DOM tree construction, style computation | Parse basic HTML, cascade styles | L2 |
| L4 | Rendering Pipeline | Framebuffer backend for FabricOS | Pixels render on bare metal | L3 |
| L5 | Text Rendering | Font loading, glyph rasterization, layout | Readable text on screen | L4 |
| L6 | Content Acquisition | HTTP client, response parsing, body extraction | Fetch example.com, display text | L5 |
| L7 | TLS + Interactive Browsing | TLS 1.3 handshake, HTTPS, scroll, URL editing | https://example.com works, user can scroll/navigate | L6, FabricOS Phase 15 |
| L8 | Window Manager Integration | FabricWindow, DisplayBackend, window events | Loom runs in windowed mode with WM syscalls 29-34 | L7, FabricOS Phase 16 |
| L9 | CSS Layout Engine | Block/flexbox layout, positioning, box model | Complex layouts render correctly | L8 |
| L10 | Image Decoding | PNG, JPEG, WebP, GIF support, LRU cache | Images display from web | L9 |
| L11 | Links & Navigation | Hit-testing, URL resolution, history, visited links | Click links, back/forward, purple visited | L10 |
| L12 | HTML Forms | Input elements, form submission, validation | Login forms work, POST data | L11 |
| L12.5 | Voice Input | Microphone toggle, waveform visualization, STT integration, voice confirmation | Voice-to-text in address bar and form fields | L12 |
| L13 | JavaScript Engine (Boa) | Boa JS integration, basic DOM bindings | JS executes, modifies DOM | L12.5 |
| L13.5 | V8 Platform Interface | `#![no_std]` V8 platform for FabricOS | Memory, threads, time, entropy services | L13 |
| L13.6 | V8 Build Integration | Compile V8 for `x86_64-unknown-none` | V8 links with FabricOS syscalls | L13.5 |
| L14 | Media Playback | Video/audio decoding, streaming | HTML5 video works | L13 |
| L15 | Security & Sandboxing | CSP, sandboxed iframes, permission UI | Secure browsing, policy enforcement | L14 |
| L16 | Accessibility | Screen reader support, ARIA, keyboard nav | WCAG 2.1 AA compliance | L15 |
| L16.5 | Hardware Media Acceleration | Platform detection, wgpu texture import, YUV shaders | 1080p60 <10% CPU, zero-copy | L16 |
| L17 | WebGL/GPU Acceleration | WebGL context, wgpu integration | 3D content renders | L16 |
| L18 | AI-Native Mode | Intent parser, agent integration, minimal chrome | AI-assisted browsing works | L17 |
| L19 | Performance Optimization | Lazy loading, virtual scrolling, caching | 60fps on complex pages | L18 |
| L20 | Servo Investigation | Research Servo WebView, hybrid engine decision | Decision: integrate Servo or continue custom | L19 |
| L21 | Traditional Mode Polish | DevTools, extensions API, CSS Grid, tab discarding | Chrome parity for complex apps | L20 |
| L22 | Daily Driver | Battery optimization, crash recovery, auto-update, onboarding | Usable as primary browser | L21 |
| L23 | V8 Integration | Replace Boa with V8 in Traditional mode, JIT compilation, performance parity with Chrome | Octane score within 20% of Chrome | L22 |
| L24 | WebAssembly | WASM runtime, compiler pipeline, memory sandboxing | Unity WebGL demo runs at 60fps | L23 |
| L25 | Service Workers | Background execution, offline apps, push notifications | PWA install, offline Gmail works | L24 |
| L26 | Full Media Codecs | H.265/HEVC, AV1 hardware, Dolby, DTS | Netflix 4K HDR playback | L25 |
| L27 | WebRTC | Peer-to-peer video, data channels, screen share | Video call with 4 participants, no drops | L26 |
| L28 | Accessibility | Screen reader APIs, full WCAG 2.1 AAA, voice navigation | Orca/NVDA integration, keyboard-only usable | L27 |

---

## AI-Native vs Traditional Mode Strategy

### AI-Native Mode (L18) — Pure & Minimal
The AI-Native mode remains **intentionally minimal and pure**:
- Intent-driven navigation ("show me flights to Tokyo")
- Concierge agent integration
- Zero chrome — content-first presentation
- Voice-first input with optional text
- Predictive page loading via Groundskeeper
- **No DevTools, no extensions, no settings clutter**

Target user: Information seekers, researchers, knowledge workers who want answers, not chrome.

### Traditional Mode (L13-L28) — Full Web Compatibility
Traditional mode gets **all daily driver features plus innovations**:
- Full web standards compliance
- Extensions (Manifest V3 subset)
- DevTools (console, inspector, network)
- Performance optimizations (battery, throttling)
- Crash recovery and auto-update
- V8 JavaScript engine with JIT
- WebAssembly support
- Service Workers and PWAs
- Full media codec stack
- WebRTC for video conferencing
- Accessibility (WCAG 2.1 AAA)

Target user: Power users, developers, enterprise, anyone needing full web compatibility.

---

## Differentiation Strategy

Loom differentiates from Chrome/Firefox/Safari through:

| Feature | Loom Approach | Chrome/Firefox |
|:---|:---|:---|
| **Zero-Copy Architecture** | Minimal data movement between engine, GPU, and AI layers | Multiple copies through compositor, JS engine, GPU process |
| **Predictive Rendering** | Groundskeeper pre-renders pages before user navigates | Reactive rendering only |
| **Capability-Scoped Security** | Permissions per capability (AI, camera, location) with hardware-backed isolation | Origin-based sandboxing |
| **Local-First Sync** | Chauffeur syncs bookmarks/history via local mesh, cloud optional | Cloud-first sync (Google/FF accounts) |
| **Native AI Integration** | First-class AI assistant, not a bolt-on extension | Extensions or side panels |
| **Dual-Mode UX** | AI-Native mode for pure browsing, Traditional for full compatibility | Single-mode, one-size-fits-all |

---

## Key Decision Points

| Decision | Phase | Status | Notes |
|:---|:---|:---|:---|
| AI-Native vs Traditional Mode Split | L2 (design) / L18 (implementation) | **Complete** | AI-Native is differentiator; Traditional is daily driver |
| JavaScript Engine | L13 | COMPLETE | Boa (Rust-native) for initial implementation |
| JavaScript Engine (Traditional) | L23 | Planned | V8 with JIT for Traditional mode performance parity |
| Servo Integration | L20 | **Decision Made** | Continue custom engine; Traditional mode gets full investment |
| Voice STT Backend | L12.5 | UNDECIDED | Local Whisper (privacy, offline) vs Web Speech API (cloud) |
| Media Codec Strategy | L14 | COMPLETE | Software decode (rav1d + symphonia), hardware acceleration L16.5 |
| Full Codec Stack | L26 | Planned | H.265/HEVC, Dolby, DTS for streaming parity |

---

## Tier 3 Completion Criteria

Per FabricOS README, Tier 3 (System Completion) requires:

1. Linux VM runs dev tools (Phase 17) — Loom runs windowed ✓
2. Gaming playable via cloud streaming (Phase 18) — WebGL support L17 ✓
3. AI marketplace live (Phase 19) — AI-Native mode L18 ✓
4. Servo decision made (Phase 20) — Continue custom engine L20 ✓

**Loom "Feature Complete" for Tier 3: L22 (Daily Driver)**

---

## Current Status

| Range | Status | Count |
|:---|:---|:---|
| L0-L16.5 | Complete | 17 phases |
| L17-L22 | **Complete** | 6 phases |
| L23-L28 | Planned, awaiting implementation | 6 phases |

Latest Completed: L22 (Daily Driver)
Next Up: L23 (V8 Integration)

---

## Workspace Crate Structure

### Current Crates

| Crate | Status | Purpose |
|:---|:---|:---|
| loom-core | Complete | Colors, geometry, text, BrowserMode, tab_manager, battery, session_restore, auto_update, onboarding |
| loom-layout | Complete | HTML/CSS/DOM, navigation, forms, hit-testing, virtual_scroll, layout_cache, lazy_image, resource_priority, css_grid |
| loom-media | Complete | Image decoding, video/audio playback, hardware acceleration |
| loom-js | Complete | JavaScript engine (Boa), DOM bindings, sandbox, gc_tuning |
| loom-security | Complete | CSP, sandboxing, permissions |
| loom-webgl | Complete | WebGL context, wgpu integration |
| loom-ai | Complete | Intent parser, chrome, agent integration, voice stub |
| loom-devtools | Complete | Console, element inspector, network monitor |
| loom-extensions | Complete | Manifest V3, content scripts, browser.tabs API |

### Planned Crates

| Crate | Phase | Purpose |
|:---|:---|:---|
| loom-v8 | L23 | V8 JavaScript engine integration (Traditional mode) |
| loom-wasm | L24 | WebAssembly runtime and compiler |
| loom-serviceworker | L25 | Service Worker background execution |
| loom-webrtc | L27 | WebRTC peer-to-peer video/data |

---

## Dependencies on FabricOS

| Loom Phase | Requires FabricOS Phase | Syscalls Used |
|:---|:---|:---|
| L4+ | Phase 10 (Display) | 18-20 (display_alloc, blit, present) |
| L6+ | Phase 11-13 (Networking) | 10-17 (socket), 22 (DNS), 24 (poll) |
| L7+ | Phase 15 (TLS) | 25-28 (TLS syscalls) |
| L8+ | Phase 16 (Window Manager) | 29-34 (WM syscalls) |
| L12.5+ | Phase 17+ (Audio) | Microphone/audio capture syscalls (future) |

---

## Success Metrics

| Phase | ORI Threshold | Key Metric |
|:---|:---|:---|
| L0-L6 | 80/100 | Renders text, fetches HTTP |
| L7-L12 | 85/100 | HTTPS works, interactive, forms functional |
| L12.5-L16 | 85/100 | Voice input works, JS executes, accessible |
| L17-L22 | 90/100 | 3D renders, AI mode works, daily driver ready |
| L23-L28 | 95/100 | Chrome parity, full web compatibility |

---

## Notes

- ORI = Operational Resilience Index (formerly OCRB)
- STRESS = System Threat Resilience & Extreme Stress Suite
- Phase numbering uses decimals (L12.5) for sub-phases
- All phases track against FabricOS Tier 3 completion
- **ROADMAP v2.0**: Extended Traditional mode scope with L23-L28 for full daily driver parity
