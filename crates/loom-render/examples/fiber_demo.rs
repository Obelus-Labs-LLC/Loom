//! Fiber Work Loop Demo
//!
//! Demonstrates time-sliced rendering with the React Fiber-style work loop.
//! Shows priority scheduling, incremental work, and 60fps guarantee.

use loom_render::fiber::{
    FiberWorkLoop, PriorityLane, incremental_fiber,
    FiberContext, FiberResult, verify_fiber_overhead,
    FpsGuarantee, TARGET_FRAME_TIME_60FPS,
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

fn main() {
    println!("=== Loom Fiber Work Loop Demo ===\n");

    // Create work loop
    let fiber_loop = FiberWorkLoop::new();

    // Counter for tracking completed work
    let items_processed = Arc::new(AtomicU64::new(0));

    // Schedule some immediate priority work (animations, input)
    println!("Scheduling work...");
    
    let processed1 = Arc::clone(&items_processed);
    fiber_loop.schedule(PriorityLane::Immediate, move |ctx: &FiberContext| {
        // Simulate animation work
        let start = Instant::now();
        while start.elapsed() < Duration::from_millis(1) {
            if ctx.should_yield() {
                return FiberResult::Continue;
            }
            std::thread::yield_now();
        }
        processed1.fetch_add(1, Ordering::Relaxed);
        FiberResult::Completed
    });

    // Schedule normal priority work (UI updates)
    let processed2 = Arc::clone(&items_processed);
    fiber_loop.schedule(PriorityLane::Normal, move |ctx: &FiberContext| {
        // Simulate UI update
        let start = Instant::now();
        while start.elapsed() < Duration::from_millis(2) {
            if ctx.should_yield() {
                return FiberResult::Continue;
            }
            std::thread::yield_now();
        }
        processed2.fetch_add(1, Ordering::Relaxed);
        FiberResult::Completed
    });

    // Schedule incremental work (large list rendering)
    let processed3 = Arc::clone(&items_processed);
    let incremental = incremental_fiber(
        1000,  // 1000 items to process
        50,    // Process 50 at a time
        move |batch| {
            processed3.fetch_add(batch.len() as u64, Ordering::Relaxed);
            // Simulate work
            std::thread::sleep(Duration::from_micros(100));
        }
    );
    fiber_loop.schedule(PriorityLane::Normal, incremental);

    // Schedule low priority work (analytics)
    let processed4 = Arc::clone(&items_processed);
    fiber_loop.schedule(PriorityLane::Low, move |ctx: &FiberContext| {
        // Background work that yields frequently
        for i in 0..100 {
            if ctx.should_yield() {
                return FiberResult::Continue;
            }
            std::thread::sleep(Duration::from_micros(50));
            processed4.fetch_add(1, Ordering::Relaxed);
        }
        FiberResult::Completed
    });

    println!("Running 10 frames of work...\n");

    // Run 10 frames
    let mut total_frame_time = Duration::ZERO;
    let mut frames_met_deadline = 0;
    let guarantee = FpsGuarantee::fps_60();

    for frame in 0..10 {
        let frame_start = Instant::now();
        
        // Run one frame
        let result = fiber_loop.run_frame();
        
        let frame_time = frame_start.elapsed();
        total_frame_time += frame_time;
        
        if result.met_deadline {
            frames_met_deadline += 1;
        }

        println!(
            "Frame {}: {:>3} completed, {:>3} yielded | Time: {:>6?} | {}",
            frame + 1,
            result.fibers_completed,
            result.fibers_yielded,
            frame_time,
            if result.met_deadline { "✓" } else { "✗" }
        );

        // Post-frame cleanup
        std::thread::sleep(Duration::from_millis(1));
    }

    // Summary
    println!("\n=== Summary ===");
    let avg_frame_time = total_frame_time / 10;
    let avg_fps = 1.0 / avg_frame_time.as_secs_f64();
    
    println!("Average frame time: {:?}", avg_frame_time);
    println!("Average FPS: {:.1}", avg_fps);
    println!("Target frame time: {:?}", TARGET_FRAME_TIME_60FPS);
    println!("Deadline hit rate: {}/10 ({}%)", frames_met_deadline, frames_met_deadline * 10);
    println!("Total items processed: {}", items_processed.load(Ordering::Relaxed));

    // Verify Fiber overhead
    let overhead_ok = verify_fiber_overhead(avg_frame_time);
    println!("\nFiber overhead check: {}", if overhead_ok { "✓ PASS" } else { "✗ FAIL" });
    println!("L29-PERF target: <2ms overhead per frame");

    // FPS Guarantee check
    let guarantee_met = guarantee.is_met(avg_frame_time);
    println!("\n60 FPS Guarantee: {}", if guarantee_met { "✓ MET" } else { "✗ MISSED" });

    println!("\n=== Demo Complete ===");
}
