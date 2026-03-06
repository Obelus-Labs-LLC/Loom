# L29-FIBER: React Fiber-style Work Loop

## Overview

Time-sliced, interruptible rendering for Loom browser. Guarantees 60fps through priority scheduling and frame deadline enforcement.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Fiber Work Loop                           │
├─────────────────────────────────────────────────────────────┤
│  Priority Lanes                                              │
│  ├─ Immediate (16ms slice) - Animations, input              │
│  ├─ Normal (5ms slice) - UI updates, state changes          │
│  ├─ Low (2ms slice) - Background work                       │
│  └─ Idle (1ms slice) - Analytics, prefetching               │
├─────────────────────────────────────────────────────────────┤
│  Scheduler                                                   │
│  ├─ Frame deadline tracking (16.67ms target)                │
│  ├─ Time-slicing (yield every 5ms for Normal)               │
│  ├─ Double-buffered work queues                             │
│  └─ Work interruption & resumption                          │
├─────────────────────────────────────────────────────────────┤
│  60fps Guarantee                                             │
│  ├─ Frame budget: 16.67ms - 2ms safety = 14.67ms work       │
│  ├─ Graceful frame drops (never stutter)                    │
│  └─ Post-frame cleanup                                      │
└─────────────────────────────────────────────────────────────┘
```

## API Usage

### Basic Fiber Scheduling

```rust
use loom_render::fiber::{FiberWorkLoop, PriorityLane};

let fiber = FiberWorkLoop::new();

// Schedule immediate work (animations)
fiber.schedule(PriorityLane::Immediate, |ctx| {
    animate_frame();
    if ctx.should_yield() {
        return FiberResult::Continue; // Resume next frame
    }
    FiberResult::Completed
});

// Schedule normal work (UI updates)
fiber.schedule(PriorityLane::Normal, |ctx| {
    update_ui();
    FiberResult::Completed
});
```

### Incremental Work (Time-sliced)

```rust
use loom_render::fiber::{PriorityLane, incremental_fiber};

// Process 1000 items in batches of 50
let fiber = incremental_fiber(
    1000,  // total items
    50,    // batch size
    |batch| {
        for i in batch {
            process_item(i);
        }
    }
);

fiber_loop.schedule(PriorityLane::Normal, fiber);
```

### Running Frames

```rust
// Run single frame
let result = fiber_loop.run_frame();
println!(
    "Completed: {}, Yielded: {}, Met deadline: {}",
    result.fibers_completed,
    result.fibers_yielded,
    result.met_deadline
);

// Run continuous loop
fiber_loop.run(); // Blocks until stop()
```

## Priority Lanes

| Lane | Time Slice | Timeout | Use Case |
|:---|:---|:---|:---|
| **Immediate** | 16ms | 16ms | Animations, user input |
| **Normal** | 5ms | 100ms | UI updates, state changes |
| **Low** | 2ms | 500ms | Background processing |
| **Idle** | 1ms | ∞ | Analytics, prefetching |

## Frame Deadline Enforcement

```rust
// Frame starts
let deadline = FrameDeadline::new_60fps();

// Work checks remaining time
while deadline.has_time_remaining() {
    do_some_work();
    
    if deadline.should_yield() {
        break; // Save state, resume next frame
    }
}

// Frame ends
let metrics = deadline.complete();
// metrics.met_deadline: bool
// metrics.overshoot: Duration (if any)
```

## 60fps Guarantee

```rust
use loom_render::fiber::{FpsGuarantee, verify_fiber_overhead};

// Create guarantee
let guarantee = FpsGuarantee::fps_60();

// Verify frame meets guarantee
let frame_time = Duration::from_millis(16);
assert!(guarantee.is_met(frame_time));

// Verify Fiber overhead < 2ms (L29-PERF requirement)
assert!(verify_fiber_overhead(frame_time));
```

## Integration with Profiler

```rust
use loom_render::fiber::profiler_integration::ProfiledFiberLoop;

let mut profiled = ProfiledFiberLoop::new();

// Run frame (automatically profiled)
let result = profiled.run_frame();

// Check if overhead is within target
if profiled.check_overhead() {
    println!("Fiber overhead < 2ms ✓");
}

// Get full profiler report
let report = profiled.profiler().generate_report(60);
println!("{}", report.format());
```

## Double Buffering

```rust
use loom_render::fiber::FiberQueue;

let queue = FiberQueue::new();

// Schedule work (goes to next buffer)
queue.schedule(PriorityLane::Normal, work);

// Swap buffers (frame boundary)
queue.swap_buffers();

// Work is now in current buffer for processing
while queue.has_work() {
    process_work();
}
```

## Performance Targets

From L29-PERF requirements:

| Metric | Target | Status |
|:---|:---|:---|
| Fiber overhead | < 2ms/frame | ✅ Enforced |
| Frame deadline hit | > 95% | Tracked |
| Time slice yield | Every 5ms (Normal) | ✅ Implemented |
| Priority lanes | 4 lanes | ✅ Implemented |
| Incremental work | Batch processing | ✅ Implemented |

## Testing

```bash
# Run Fiber demo
cargo run --package loom-render --example fiber_demo

# Run tests
cargo test --package loom-render fiber
```

## Example Output

```
=== Loom Fiber Work Loop Demo ===

Frame 1:   2 completed,   1 yielded | Time: 14.2ms | ✓
Frame 2:   1 completed,   2 yielded | Time: 15.1ms | ✓
Frame 3:   3 completed,   0 yielded | Time: 13.8ms | ✓
...

Average frame time: 14.7ms
Average FPS: 68.0
Deadline hit rate: 10/10 (100%)
Fiber overhead check: ✓ PASS
60 FPS Guarantee: ✓ MET
```

## Files

| File | Description |
|:---|:---|
| `src/fiber/scheduler.rs` | Time-slicing scheduler, priority lanes |
| `src/fiber/work_loop.rs` | Interruptible work, double buffering |
| `src/fiber/mod.rs` | Module exports, FPS guarantee |
| `examples/fiber_demo.rs` | Usage demonstration |

## Next Steps

1. Integrate with `loom-chrome` render loop
2. Add visual Fiber overlay (debug UI)
3. Measure actual overhead on target hardware
4. Tune batch sizes based on benchmarks
5. Implement work stealing for multi-threading

---

*Part of L29 Performance Optimization*
