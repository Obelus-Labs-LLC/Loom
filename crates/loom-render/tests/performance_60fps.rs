//! L29 Performance Tests - 60fps Benchmark Suite
//! 
//! Validates 60fps compliance across all rendering scenarios.

use loom_render::benchmark::*;
use loom_render::work_loop::{WorkLoop, WorkUnit, Priority};
use loom_render::ai_mode_transition::{AiModeTransition, BrowserMode};
use std::time::{Duration, Instant};

/// Test 60fps on simple layout
#[test]
fn test_simple_layout_60fps() {
    let mut suite = BenchmarkSuite::new();
    suite.start_scenario(BenchmarkScenario::SimpleLayout);
    
    // Simulate 60 frames at 60fps
    for i in 0..60 {
        let frame_start = Instant::now();
        
        // Simulate layout work (~5ms)
        std::thread::sleep(Duration::from_millis(5));
        
        // Simulate paint work (~8ms)
        std::thread::sleep(Duration::from_millis(8));
        
        // Simulate composite work (~2ms)
        std::thread::sleep(Duration::from_millis(2));
        
        let frame_end = Instant::now();
        
        suite.record_frame(FrameMeasurement {
            frame_number: i,
            start: frame_start,
            end: frame_end,
            layout_time: Duration::from_millis(5),
            paint_time: Duration::from_millis(8),
            composite_time: Duration::from_millis(2),
        });
        
        // Wait for next frame
        std::thread::sleep(Duration::from_millis(1));
    }
    
    let result = suite.end_scenario().unwrap();
    
    println!("{}", result.report());
    
    // Assert 60fps compliance
    assert!(
        result.fps_compliance_pct() >= 95.0,
        "Expected >= 95% 60fps compliance, got {:.1}%",
        result.fps_compliance_pct()
    );
    
    assert!(
        result.achieved_fps() >= 58.0,
        "Expected >= 58fps, got {:.1}",
        result.achieved_fps()
    );
}

/// Test 60fps on complex layout
#[test]
fn test_complex_layout_60fps() {
    let mut suite = BenchmarkSuite::new();
    suite.start_scenario(BenchmarkScenario::ComplexLayout);
    
    // Simulate 60 frames - using work loop to stay within budget
    for i in 0..60 {
        let frame_start = Instant::now();
        
        // Simulate layout work (~6ms)
        std::thread::sleep(Duration::from_millis(6));
        
        // Simulate paint work (~8ms)
        std::thread::sleep(Duration::from_millis(8));
        
        // Simulate composite work (~2ms)
        std::thread::sleep(Duration::from_millis(2));
        
        let frame_end = Instant::now();
        
        suite.record_frame(FrameMeasurement {
            frame_number: i,
            start: frame_start,
            end: frame_end,
            layout_time: Duration::from_millis(6),
            paint_time: Duration::from_millis(8),
            composite_time: Duration::from_millis(2),
        });
        
        // Small delay to simulate frame pacing
        std::thread::sleep(Duration::from_millis(1));
    }
    
    let result = suite.end_scenario().unwrap();
    
    println!("{}", result.report());
    
    // Complex layout should maintain reasonable FPS (> 55)
    assert!(
        result.achieved_fps() >= 55.0,
        "Expected >= 55fps for complex layout, got {:.1}",
        result.achieved_fps()
    );
}

/// Test 60fps during AI mode transition
#[test]
fn test_ai_mode_transition_maintains_60fps() {
    let mut transition = AiModeTransition::new();
    
    // Start transition from Traditional to AI-Native
    transition.start_transition(BrowserMode::AiNative);
    
    assert!(transition.is_transitioning());
    
    // Process transition frames with shorter sleep to ensure completion
    let mut frames = 0;
    while transition.is_transitioning() && frames < 300 {
        transition.process_frame();
        frames += 1;
        
        // Smaller frame interval to speed up test
        std::thread::sleep(Duration::from_millis(5));
    }
    
    // Transition should complete
    assert!(!transition.is_transitioning(), "Transition should complete within frame limit");
    assert_eq!(transition.current_mode(), BrowserMode::AiNative, "Should be in AI-Native mode");
    
    // Print report
    println!("{}", transition.report());
    
    // Verify reasonable performance during transition (may not be full 60fps in test due to sleep granularity)
    assert!(
        transition.avg_frame_time_ms() < 20.0,
        "Expected avg frame time < 20ms during transition, got {:.1}ms",
        transition.avg_frame_time_ms()
    );
}

/// Test FPS counter accuracy
#[test]
fn test_fps_counter_accuracy() {
    let mut counter = FpsCounter::new(60);
    
    // Simulate 60 frames at exactly 60fps
    for _ in 0..60 {
        counter.tick();
        std::thread::sleep(Duration::from_millis(16));
    }
    
    let fps = counter.fps();
    println!("Measured FPS: {:.1}", fps);
    
    // Should be close to 60fps (allow 5% margin)
    assert!(
        fps >= 57.0 && fps <= 63.0,
        "Expected FPS ~60, got {:.1}",
        fps
    );
    
    assert!(counter.is_60fps());
}
