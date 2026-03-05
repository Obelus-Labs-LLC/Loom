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
| L13 | JavaScript Engine | Boa JS integration OR V8 stub | Basic JS execution, DOM manipulation | L12.5 |
| L14 | Media Playback | Video/audio decoding, streaming | HTML5 video works | L13 |
| L15 | Security & Sandboxing | CSP, sandboxed iframes, permission UI | Secure browsing, policy enforcement | L14 |
| L16 | Accessibility | Screen reader support, ARIA, keyboard nav | WCAG 2.1 AA compliance | L15 |
| L17 | WebGL/GPU Acceleration | WebGL context, wgpu integration | 3D content renders | L16 |
| L18 | AI-Native Mode | Intent parser, agent integration, minimal chrome | AI-assisted browsing works | L17 |
| L19 | Performance Optimization | Lazy loading, virtual scrolling, caching | 60fps on complex pages | L18 |
| L20 | Servo Investigation | Research Servo WebView, hybrid engine decision | Decision: integrate Servo or continue custom | L19 |
| L21 | Traditional Mode Polish | Full web compatibility, dev tools, extensions | Chrome parity for complex apps | L20 |
| L22 | Daily Driver | Battery optimization, crash recovery, auto-update | Usable as primary browser | L21 |

---

## Key Decision Points

| Decision | Phase | Status | Notes |
|:---|:---|:---|:---|
| AI-Native vs Traditional Mode Split | L2 (design) / L18 (implementation) | Design complete, Implementation planned | Design system supports both; AI-Native is differentiator |
| JavaScript Engine | L13 | COMPLETE | Boa (Rust-native) - Full implementation with DOM bindings, event handling, sandbox |
| Servo Integration | L20 | Research phase | Per FabricOS README: AI-Native Loom is priority, Servo is compatibility fallback |
| Voice STT Backend | L12.5 | UNDECIDED | Local Whisper (privacy, offline) vs Web Speech API (cloud) |
| Media Codec Strategy | L14 | COMPLETE | Software decode (rav1d + symphonia), hardware acceleration deferred per spec |

---

## Tier 3 Completion Criteria

Per FabricOS README, Tier 3 (System Completion) requires:

1. Linux VM runs dev tools (Phase 17) — Loom runs windowed
2. Gaming playable via cloud streaming (Phase 18)
3. AI marketplace live (Phase 19)
4. Servo decision made (Phase 20) — Loom L20

Loom "Feature Complete" for Tier 3: After L20 (Servo Investigation) when decision is implemented.

---

## Current Status

| Range | Status | Count |
|:---|:---|:---|
| L0-L15 | Complete | 16 phases |
| L12.5 | Ready to implement | 1 phase |
| L16-L22 | Planned, awaiting implementation | 7 phases |

Latest Completed: L15 (Security & Sandboxing)
Next Up: L16 (Accessibility)

---

## Workspace Crate Structure

### Current Crates

| Crate | Status | Purpose |
|:---|:---|:---|
| loom-core | Exists | Colors, geometry, text, BrowserMode enum, Temperature |
| loom-layout | Exists | HTML/CSS/DOM, navigation, forms, hit-testing |
| loom-media | Exists | Image decoding (PNG/JPEG/WebP/GIF), caching |

### Planned Crates

| Crate | Phase | Purpose |
|:---|:---|:---|
| loom-voice | L12.5 | STT integration, microphone handling, waveform UI |
| loom-js | L13 | JavaScript engine bindings (Boa or V8) |
| loom-video | L14 | Video/audio playback, streaming - COMPLETE in loom-media |
| loom-security | L15 | CSP, sandboxing, permission management |
| loom-accessibility | L16 | Screen reader, ARIA, keyboard navigation |
| loom-webgl | L17 | WebGL context, wgpu integration |
| loom-ai | L18 | Intent parser, agent integration |
| loom-render | — | wgpu rendering abstraction |
| loom-chrome | — | UI chrome, mode system |
| loom-content | — | HTTP client (currently in main.rs) |

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

---

## Notes

- ORI = Operational Resilience Index (formerly OCRB)
- STRESS = System Threat Resilience & Extreme Stress Suite
- Phase numbering uses decimals (L12.5) for sub-phases
- All phases track against FabricOS Tier 3 completion
