# Loom Browser

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

A Rust-native browser built for the [FabricOS](https://github.com/Obelus-Labs-LLC/FabricOS) microkernel. Loom features dual-mode operation (Traditional and AI-assisted), a unique chromatic temperature system, and direct kernel integration via syscalls.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Loom Browser                          │
├─────────────────────────────────────────────────────────────┤
│  Chrome Layer    │  Content Layer    │  Media Layer         │
│  ├─ Mode system  │  ├─ HTTP client   │  ├─ Video decode    │
│  ├─ Tab bar      │  ├─ HTML parser   │  ├─ Audio decode    │
│  └─ Address bar  │  └─ CSS parser    │  └─ Streaming       │
├─────────────────────────────────────────────────────────────┤
│  Layout Engine           │  Render Engine                    │
│  ├─ DOM tree             │  ├─ wgpu (desktop)               │
│  ├─ Style computation    │  └─ Framebuffer (FabricOS)        │
│  └─ Box layout           │                                   │
├─────────────────────────────────────────────────────────────┤
│                    Platform Abstraction                      │
│  ├─ Desktop: wgpu + winit  │  FabricOS: Kernel syscalls     │
└─────────────────────────────────────────────────────────────┘
```

## Features

### Dual Mode System
- **Traditional Mode**: Dense, familiar browser UI with tabs, address bar, and navigation controls
- **AI-Assisted Mode**: Spacious, intent-driven interface with floating input and minimal chrome

### Design System
- **Chromatic Temperature**: UI adapts to time of day (Warm/Amber morning, Neutral afternoon, Cool/Indigo evening)
- **Tension Curves**: Organic, bezier-based shapes instead of rounded rectangles
- **Typography**: Tension Sans (UI), Weave Serif (content), Fabric Mono (data)

### Platform Support
| Platform | Backend | Status |
|----------|---------|--------|
| Windows/Linux | wgpu + winit | ✅ Working |
| FabricOS | Kernel syscalls | ✅ Ready |

## Building

### Prerequisites
- Rust 1.75+ with nightly toolchain
- For desktop: Windows or Linux with graphics drivers
- For FabricOS: See [FabricOS build instructions](https://github.com/Obelus-Labs-LLC/FabricOS)

### Desktop Build
```bash
# Standard desktop build (Windows/Linux)
cargo run

# Release build
cargo run --release
```

### FabricOS Build
```bash
# Install nightly toolchain
rustup toolchain install nightly
rustup component add rust-src --toolchain nightly

# Build for FabricOS (no_std, direct syscalls)
cargo +nightly build --release -Z build-std=core,compiler_builtins,alloc --no-default-features

# Copy to FabricOS initramfs
cp target/x86_64-unknown-none/release/loom ../FabricOS/initramfs/bin/
```

## FabricOS Integration

Loom runs as a userspace process on FabricOS, using direct syscalls for all operations:

### Socket Syscalls (Phase 9)
| Syscall | Number | Description |
|---------|--------|-------------|
| `sys_socket` | 10 | Create socket |
| `sys_bind` | 11 | Bind to address |
| `sys_listen` | 12 | Listen for connections |
| `sys_accept` | 13 | Accept connection |
| `sys_connect` | 14 | Connect to remote |
| `sys_send` | 15 | Send data |
| `sys_recv` | 16 | Receive data |
| `sys_shutdown` | 17 | Shutdown socket |

### Display Syscalls (Phase 10)
| Syscall | Number | Description |
|---------|--------|-------------|
| `sys_display_alloc` | 18 | Allocate a display surface |
| `sys_display_blit` | 19 | Blit pixel buffer to surface |
| `sys_display_present` | 20 | Present surface to screen |

### Input Syscalls (Phase 11)
| Syscall | Number | Description |
|---------|--------|-------------|
| `sys_kb_read` | 21 | Read keyboard input |

### DNS Syscalls (Phase 12)
| Syscall | Number | Description |
|---------|--------|-------------|
| `sys_dns_resolve` | 22 | Resolve hostname to IPv4 |

### Poll Syscalls (Phase 13)
| Syscall | Number | Description |
|---------|--------|-------------|
| `sys_poll` | 24 | Poll for I/O events |

### TLS Syscalls (Phase 15)
| Syscall | Number | Description |
|---------|--------|-------------|
| `sys_tls_connect` | 25 | TLS handshake |
| `sys_tls_send` | 26 | Send encrypted data |
| `sys_tls_recv` | 27 | Receive encrypted data |
| `sys_tls_close` | 28 | Close TLS session |

### Window Manager Syscalls (Phase 16)
| Syscall | Number | Description |
|---------|--------|-------------|
| `sys_wm_create` | 29 | Create window |
| `sys_wm_destroy` | 30 | Destroy window |
| `sys_wm_blit` | 31 | Blit to window |
| `sys_wm_move_resize` | 32 | Move/resize window |
| `sys_wm_focus` | 33 | Focus window |
| `sys_wm_event` | 34 | Poll window event |

## Project Structure

```
loom/
├── Cargo.toml              # Workspace manifest
├── link.x                  # Linker script for FabricOS
├── src/
│   ├── main.rs            # Entry point with platform abstraction
│   ├── lib.rs             # Library exports
│   └── os/
│       ├── mod.rs         # OS abstraction
│       └── fabricsys.rs   # FabricOS syscall interface
├── crates/
│   ├── loom-core/         # Core types, colors, geometry, text
│   ├── loom-layout/       # HTML/CSS parsing, layout engine, navigation
│   ├── loom-media/        # Image decoding (PNG, JPEG, WebP), caching
│   └── loom-design/       # Design system, temperature, typography
└── design/                # Figma exports, design tokens
```

### Crate Details

| Crate | Description | Key Modules |
|-------|-------------|-------------|
| `loom-core` | Shared primitives | `color`, `geometry`, `text`, `BrowserMode` |
| `loom-layout` | Layout engine + navigation + forms | `css_types`, `layout_engine`, `navigation`, `hittest`, `dom`, `forms` |
| `loom-media` | Media decoding | `image` (PNG, JPEG, WebP, GIF), `ImageCache`, `ResponsiveImage` |
| `loom-design` | Design system | `temperature`, `typography`, `tension_curves` |

## Design System

### Chromatic Temperature
Colors adapt based on time of day, not function:

```rust
// Warm (Amber) - Morning
WARM_50:  Oklch::new(0.98, 0.01, 60.0)  // Background
WARM_500: Oklch::new(0.68, 0.15, 40.0)  // Accent

// Cool (Indigo) - Evening
COOL_50:  Oklch::new(0.98, 0.01, 240.0) // Background
COOL_500: Oklch::new(0.58, 0.11, 240.0) // Accent

// Neutral (Silver) - Afternoon
NEUTRAL_50:  Oklch::new(0.98, 0.005, 0.0) // Background
NEUTRAL_500: Oklch::new(0.58, 0.005, 0.0) // Accent
```

### Tension Curves
Organic shapes with bezier-based corners:

```rust
let rect = TensionRect::new(x, y, width, height)
    .with_tension(0.15); // Organic feel
```

## Current Status

| Phase | Feature | Status |
|-------|---------|--------|
| L0 | Bootstrap, window opening | ✅ Complete |
| L1 | wgpu surface, 60fps | ✅ Complete |
| L2 | Design system implementation | ✅ Complete |
| L3 | HTML/CSS parsing | ✅ Complete |
| L4 | Rendering pipeline | ✅ Complete (framebuffer) |
| L5 | Text rendering, layout | ✅ Complete |
| L6 | Content acquisition | ✅ Complete |
| L7 | TLS 1.3 + Interactive browsing | ✅ Complete |
| L8 | Window Manager integration | ✅ Complete |
| L9 | CSS Layout Engine | ✅ Complete |
| L10 | Image decoding (PNG/JPEG/WebP) | ✅ Complete |
| L11 | Links and Navigation | ✅ Complete |
| L12 | HTML Forms | ✅ Complete |
| L13 | JavaScript engine | 📋 Planned |
| L13 | Media playback | 📋 Planned |
| L14 | Security integration | 📋 Planned |
| L15 | Polish, daily driver | 📋 Planned |

## Testing on FabricOS

```bash
# 1. Build Loom for FabricOS
cd ~/Projects/Loom
cargo +nightly build --release -Z build-std=core,compiler_builtins,alloc --no-default-features

# 2. Copy to initramfs
cp target/x86_64-unknown-none/release/loom ../FabricOS/initramfs/bin/

# 3. Build FabricOS ISO
cd ../FabricOS
make iso

# 4. Run in QEMU
make run
```

Expected output:
- FabricOS boots
- Loom spawns as userspace process in a window
- Window Manager renders title bar and decorations
- Interactive browsing: scroll, URL editing, navigation
- HTTPS support: TLS 1.3 handshake with example.com
- Window can be moved, resized, focused with Alt+Tab

## License

Loom is licensed under the GNU General Public License v3.0 (GPL-3.0). See [LICENSE](LICENSE) for details.

This license aligns with the FabricOS kernel philosophy: free software that respects user sovereignty and ensures the browser remains open and auditable.

## Related Projects

- [FabricOS](https://github.com/Obelus-Labs-LLC/FabricOS) - AI-coordinated microkernel
- [Loom Design](design/) - Figma exports and design tokens

---

Built with 🦀 Rust for 🔬 Obelus Labs
