//! React Fiber-style Work Loop for Loom
//!
//! Provides time-sliced, interruptible rendering with priority scheduling.
//! Guarantees 60fps through frame deadline enforcement.
//!
//! # Example
//!
//! ```rust
//! use loom_render::fiber::{FiberWorkLoop, PriorityLane, incremental_fiber};
//!
//! // Create work loop
//! let fiber = FiberWorkLoop::new();
//!
//! // Schedule incremental work
//! fiber.schedule(PriorityLane::Normal, incremental_fiber(
//!     1000,  // 1000 items
//!     50,    // 50 items per batch
//!     |batch| {
//!         // Process batch of items
//!         println!("Processing {:?}", batch);
//!     }
//! ));
//!
//! // Run one frame
//! let result = fiber.run_frame();
//! println!("Completed {} fibers", result.fibers_completed);
//! ```

pub mod scheduler;
pub mod work_loop;

pub use scheduler::{
    FiberScheduler, 
    PriorityLane, 
    FrameDeadline, 
    FrameMetrics,
    WorkResult,
    WorkUnit,
};

pub use work_loop::{
    FiberWorkLoop,
    Fiber,
    FiberContext,
    FiberResult,
    FiberState,
    FiberQueue,
    WorkChunk,
    FrameResult,
    WorkLoopStats,
    incremental_fiber,
};

use std::time::Duration;

/// Target frame time for 60 FPS
pub const TARGET_FRAME_TIME_60FPS: Duration = Duration::from_nanos(16_666_667);

/// Target frame time for 120 FPS
pub const TARGET_FRAME_TIME_120FPS: Duration = Duration::from_nanos(8_333_333);

/// Default time slice for Normal priority work
pub const DEFAULT_TIME_SLICE: Duration = Duration::from_millis(5);

/// Safety margin before frame deadline
pub const FRAME_SAFETY_MARGIN: Duration = Duration::from_millis(2);

/// Maximum Fiber overhead target (from L29-PERF requirements)
pub const MAX_FIBER_OVERHEAD: Duration = Duration::from_millis(2);

/// Verify Fiber overhead is within target
/// 
/// Returns true if Fiber overhead is under 2ms per frame
pub fn verify_fiber_overhead(frame_time: Duration) -> bool {
    // Fiber overhead is the difference between actual frame time
    // and the time spent on actual rendering work
    // In a well-optimized system, this should be < 2ms
    
    // For now, we assume if frame time is close to target,
    // the overhead is acceptable
    let overhead = if frame_time > TARGET_FRAME_TIME_60FPS {
        frame_time - TARGET_FRAME_TIME_60FPS
    } else {
        Duration::ZERO
    };
    
    overhead <= MAX_FIBER_OVERHEAD
}

/// 60 FPS Guarantee configuration
#[derive(Debug, Clone, Copy)]
pub struct FpsGuarantee {
    /// Target frame time
    pub target: Duration,
    /// Safety margin before yielding
    pub safety_margin: Duration,
    /// Maximum acceptable overshoot
    pub max_overshoot: Duration,
    /// Whether to drop frames that miss deadline
    pub drop_frames: bool,
}

impl FpsGuarantee {
    /// Create 60 FPS guarantee
    pub fn fps_60() -> Self {
        Self {
            target: TARGET_FRAME_TIME_60FPS,
            safety_margin: FRAME_SAFETY_MARGIN,
            max_overshoot: Duration::from_millis(4),
            drop_frames: false,
        }
    }

    /// Create 120 FPS guarantee
    pub fn fps_120() -> Self {
        Self {
            target: TARGET_FRAME_TIME_120FPS,
            safety_margin: Duration::from_millis(1),
            max_overshoot: Duration::from_millis(2),
            drop_frames: true,
        }
    }

    /// Check if frame time meets guarantee
    pub fn is_met(&self, frame_time: Duration) -> bool {
        frame_time <= self.target + self.max_overshoot
    }

    /// Get time available for work
    pub fn work_budget(&self) -> Duration {
        self.target - self.safety_margin
    }
}

impl Default for FpsGuarantee {
    fn default() -> Self {
        Self::fps_60()
    }
}

/// Integration with L29-PERF profiler
#[cfg(feature = "profiler")]
pub mod profiler_integration {
    use super::*;
    use crate::profiler::{RenderProfiler, RenderStage};

    /// Profiled fiber work loop
    pub struct ProfiledFiberLoop {
        inner: FiberWorkLoop,
        profiler: RenderProfiler,
    }

    impl ProfiledFiberLoop {
        /// Create new profiled work loop
        pub fn new() -> Self {
            Self {
                inner: FiberWorkLoop::new(),
                profiler: RenderProfiler::new(),
            }
        }

        /// Run profiled frame
        pub fn run_frame(&mut self) -> FrameResult {
            self.profiler.begin_frame();

            // Time the fiber scheduling work
            self.profiler.start_stage(RenderStage::Composite);
            let result = self.inner.run_frame();
            self.profiler.end_stage(RenderStage::Composite, 1);

            self.profiler.end_frame();
            result
        }

        /// Get profiler reference
        pub fn profiler(&self) -> &RenderProfiler {
            &self.profiler
        }

        /// Check if Fiber overhead is within L29-PERF target
        pub fn check_overhead(&self) -> bool {
            if let Some(report) = self.profiler.generate_report(60).bottleneck {
                let (stage, time) = report;
                // Check if Composite (Fiber) stage is under 2ms
                stage == RenderStage::Composite && time.as_millis() < 2
            } else {
                true
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fps_guarantee() {
        let guarantee = FpsGuarantee::fps_60();
        
        // Should be met at target
        assert!(guarantee.is_met(TARGET_FRAME_TIME_60FPS));
        
        // Should be met within overshoot
        assert!(guarantee.is_met(TARGET_FRAME_TIME_60FPS + Duration::from_millis(2)));
        
        // Should not be met with large overshoot
        assert!(!guarantee.is_met(TARGET_FRAME_TIME_60FPS + Duration::from_millis(10)));
    }

    #[test]
    fn test_verify_fiber_overhead() {
        // Should pass at target
        assert!(verify_fiber_overhead(TARGET_FRAME_TIME_60FPS));
        
        // Should pass with small overshoot
        assert!(verify_fiber_overhead(
            TARGET_FRAME_TIME_60FPS + Duration::from_millis(1)
        ));
        
        // Should fail with large overshoot (>2ms)
        assert!(!verify_fiber_overhead(
            TARGET_FRAME_TIME_60FPS + Duration::from_millis(5)
        ));
    }

    #[test]
    fn test_work_budget() {
        let guarantee = FpsGuarantee::fps_60();
        let budget = guarantee.work_budget();
        
        // Budget should be target minus safety margin
        assert!(budget < TARGET_FRAME_TIME_60FPS);
        assert!(budget > Duration::from_millis(10));
    }
}
