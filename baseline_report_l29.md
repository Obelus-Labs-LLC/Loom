# L29 Performance Baseline Report (Template)

**Generated:** 2026-03-06  
**Loom Version:** 0.1.0-L29  
**Status:** 🚧 Pending actual measurements

---

## Quick Start

Run the baseline benchmark:

```bash
cd /mnt/c/Users/dshon/Projects/Loom
cargo run --package loom-bench --example baseline
```

---

## Current Implementation Status

| Component | Status | File |
|:---|:---|:---|
| FPS Counter | ✅ Complete | `crates/loom-bench/src/fps_counter.rs` |
| Frame Histogram (p50/p95/p99) | ✅ Complete | `crates/loom-bench/src/fps_counter.rs` |
| GPU/CPU Split Tracking | ✅ Complete | `crates/loom-bench/src/fps_counter.rs` |
| Render Pipeline Profiler | ✅ Complete | `crates/loom-render/src/profiler.rs` |
| Per-Stage Timing | ✅ Complete | `crates/loom-render/src/profiler.rs` |
| Memory Allocation Tracking | ✅ Complete | `crates/loom-render/src/profiler.rs` |
| Bottleneck Detection | ✅ Complete | `crates/loom-render/src/profiler.rs` |
| Baseline Report Generator | ✅ Complete | `crates/loom-bench/src/baseline.rs` |
| Test Page Suite | ✅ Complete | `crates/loom-bench/src/lib.rs` |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    L29 Performance Layer                     │
├─────────────────────────────────────────────────────────────┤
│  loom-bench                                                  │
│  ├─ FpsCounter: Rolling FPS, frame histograms               │
│  ├─ FrameStats: p50, p95, p99, mean, std_dev                │
│  ├─ SplitTime: GPU vs CPU time tracking                     │
│  └─ BaselineGenerator: Report generation                    │
├─────────────────────────────────────────────────────────────┤
│  loom-render                                                 │
│  ├─ RenderProfiler: Per-stage timing                        │
│  ├─ RenderStage: Style, Layout, Paint, Raster, GPU          │
│  ├─ StageTiming: Duration + element count                   │
│  └─ Memory tracking: Per-frame allocations                  │
├─────────────────────────────────────────────────────────────┤
│  Integration Points                                          │
│  ├─ Chrome: Frame timer integration                         │
│  ├─ Render: ProfiledFrame RAII wrapper                      │
│  └─ WebGL: GPU time queries                                 │
└─────────────────────────────────────────────────────────────┘
```

---

## Test Pages

Three standard test pages for consistent benchmarking:

| Page | Elements | Complexity | Purpose |
|:---|:---|:---|:---|
| `simple_static` | 10 | Low | Baseline measurement |
| `complex_layout` | 50 | Medium | Layout engine stress |
| `stress_test` | 1005 | High | 60 FPS boundary test |

---

## Key Metrics

### FPS Counter
- **Rolling window**: 120 frames (2 seconds at 60 FPS)
- **History depth**: 1000 frames
- **Percentiles**: p50 (median), p95, p99
- **Target tracking**: Hit rate % for 60 FPS

### Render Profiler
- **Stages tracked**: 7 (Style, Layout, Paint, Raster, Composite, GPU, Present)
- **Memory tracking**: Per-category allocation stats
- **Bottleneck detection**: Automatic slowest stage identification

---

## Next Steps

1. **Integrate with loom-chrome**: Add FPS display in UI
2. **Integrate with loom-render**: Wrap render loop with profiler
3. **Run baseline**: Generate actual measurements on target hardware
4. **Identify bottlenecks**: Analyze p95/p99 outliers
5. **L29-FIBER**: Implement React Fiber work loop based on findings

---

## Expected Targets (Pre-Measurement)

| Page | Target FPS | Minimum FPS | Notes |
|:---|:---|:---|:---|
| simple_static | 60+ | 60 | Should be trivial |
| complex_layout | 60 | 45 | Flexbox, nesting |
| stress_test | 30 | 15 | Virtual scroll needed |

---

## API Usage Example

```rust
use loom_bench::fps_counter::FpsCounter;
use loom_render::profiler::{RenderProfiler, RenderStage, ProfiledFrame};

// FPS tracking
let mut fps = FpsCounter::new();

loop {
    fps.begin_frame();
    
    // CPU work
    layout_engine.calculate();
    fps.begin_gpu();
    
    // GPU work
    renderer.draw();
    
    let measurement = fps.end_frame();
    println!("Frame {}: {:?} (CPU: {:?}, GPU: {:?})",
        measurement.frame_num,
        measurement.total_time,
        measurement.cpu_time,
        measurement.gpu_time);
}

// Generate report
let stats = fps.frame_stats();
println!("p50: {:?}, p95: {:?}, p99: {:?}",
    stats.p50, stats.p95, stats.p99);
```

```rust
use loom_render::profiler::{RenderProfiler, RenderStage, ProfiledFrame};

let mut profiler = RenderProfiler::new();

// Profile a frame
{
    let mut frame = ProfiledFrame::new(&mut profiler);
    
    // Profile individual stages
    {
        let _stage = frame.stage(RenderStage::Layout);
        layout_engine.calculate();
    }
    
    {
        let _stage = frame.stage(RenderStage::Paint);
        paint_engine.generate_commands();
    }
}

// Generate report
let report = profiler.generate_report(60);
println!("{}", report.format());
```

---

*This is a template report. Actual measurements will be generated after integration with the renderer.*
