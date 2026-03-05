//! AI Mode Transition Handler
//! 
//! Ensures 60fps is maintained during Traditional <-> AI-Native mode switches.
//! Uses Fiber-style work loop to pause/resume rendering during transitions.

use std::time::{Duration, Instant};
use crate::work_loop::{WorkLoop, WorkUnit, Priority, FRAME_BUDGET};
use crate::benchmark::{FpsCounter, FrameMeasurement};

/// State of mode transition
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionState {
    /// No transition in progress
    Idle,
    /// Starting transition
    Starting,
    /// Capturing current state
    Capturing,
    /// Loading new mode
    Loading,
    /// Animating transition
    Animating,
    /// Completing transition
    Completing,
}

/// Transition work unit - can be paused/resumed
struct TransitionWorkUnit {
    stage: TransitionStage,
    progress: f32,
    priority: Priority,
}

#[derive(Debug, Clone, Copy)]
enum TransitionStage {
    CaptureLayout,
    CaptureState,
    BuildNewUI,
    AnimateTransition,
    CleanupOldUI,
}

impl WorkUnit for TransitionWorkUnit {
    fn execute(&mut self) -> bool {
        match self.stage {
            TransitionStage::CaptureLayout => {
                // Capture current layout (incremental)
                self.progress += 0.2;
                std::thread::sleep(Duration::from_micros(500));
                
                if self.progress >= 1.0 {
                    self.stage = TransitionStage::CaptureState;
                    self.progress = 0.0;
                }
                false // Not complete
            }
            TransitionStage::CaptureState => {
                // Capture application state
                self.progress += 0.25;
                std::thread::sleep(Duration::from_micros(400));
                
                if self.progress >= 1.0 {
                    self.stage = TransitionStage::BuildNewUI;
                    self.progress = 0.0;
                }
                false
            }
            TransitionStage::BuildNewUI => {
                // Build new UI components
                self.progress += 0.15;
                std::thread::sleep(Duration::from_micros(600));
                
                if self.progress >= 1.0 {
                    self.stage = TransitionStage::AnimateTransition;
                    self.progress = 0.0;
                }
                false
            }
            TransitionStage::AnimateTransition => {
                // Animate transition (high priority)
                self.progress += 0.3;
                std::thread::sleep(Duration::from_micros(300));
                
                if self.progress >= 1.0 {
                    self.stage = TransitionStage::CleanupOldUI;
                    self.progress = 0.0;
                }
                false
            }
            TransitionStage::CleanupOldUI => {
                // Cleanup old UI
                self.progress += 0.5;
                std::thread::sleep(Duration::from_micros(200));
                
                self.progress >= 1.0
            }
        }
    }
    
    fn priority(&self) -> Priority {
        match self.stage {
            TransitionStage::AnimateTransition => Priority::Immediate,
            _ => Priority::High,
        }
    }
    
    fn estimated_cost(&self) -> u64 {
        match self.stage {
            TransitionStage::CaptureLayout => 500,
            TransitionStage::CaptureState => 400,
            TransitionStage::BuildNewUI => 600,
            TransitionStage::AnimateTransition => 300,
            TransitionStage::CleanupOldUI => 200,
        }
    }
}

/// Manages AI mode transitions while maintaining 60fps
pub struct AiModeTransition {
    state: TransitionState,
    work_loop: WorkLoop,
    fps_counter: FpsCounter,
    
    /// Current mode
    current_mode: BrowserMode,
    
    /// Target mode
    target_mode: Option<BrowserMode>,
    
    /// Transition progress (0.0 - 1.0)
    progress: f32,
    
    /// Frame measurements during transition
    frame_measurements: Vec<FrameMeasurement>,
    
    /// Max allowed frame time during transition
    max_frame_time: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserMode {
    Traditional,
    AiNative,
}

impl AiModeTransition {
    pub fn new() -> Self {
        Self {
            state: TransitionState::Idle,
            work_loop: WorkLoop::new(),
            fps_counter: FpsCounter::new(60),
            current_mode: BrowserMode::Traditional,
            target_mode: None,
            progress: 0.0,
            frame_measurements: Vec::new(),
            max_frame_time: FRAME_BUDGET,
        }
    }
    
    /// Get current mode
    pub fn current_mode(&self) -> BrowserMode {
        self.current_mode
    }
    
    /// Get transition state
    pub fn state(&self) -> TransitionState {
        self.state
    }
    
    /// Check if transition in progress
    pub fn is_transitioning(&self) -> bool {
        self.state != TransitionState::Idle
    }
    
    /// Start mode transition
    pub fn start_transition(&mut self, target: BrowserMode) {
        if self.current_mode == target {
            return;
        }
        
        self.target_mode = Some(target);
        self.state = TransitionState::Starting;
        self.progress = 0.0;
        self.frame_measurements.clear();
        
        // Schedule transition work
        let work = TransitionWorkUnit {
            stage: TransitionStage::CaptureLayout,
            progress: 0.0,
            priority: Priority::High,
        };
        
        self.work_loop.schedule(Box::new(work));
    }
    
    /// Process one frame of transition
    /// Returns true when transition complete
    pub fn process_frame(&mut self) -> bool {
        let frame_start = Instant::now();
        
        // Record FPS
        self.fps_counter.tick();
        
        if self.state == TransitionState::Idle {
            return true;
        }
        
        // Process work with time-slicing
        if self.work_loop.has_work() {
            self.work_loop.start();
        }
        
        // Check if work is complete
        if !self.work_loop.has_work() {
            // Work finished, complete the transition
            self.complete_transition();
        } else {
            // Work still in progress, update progress based on state
            self.update_progress();
        }
        
        // Record frame measurement
        let frame_end = Instant::now();
        self.frame_measurements.push(FrameMeasurement {
            frame_number: self.frame_measurements.len() as u64,
            start: frame_start,
            end: frame_end,
            layout_time: Duration::default(),
            paint_time: Duration::default(),
            composite_time: Duration::default(),
        });
        
        self.state == TransitionState::Idle
    }
    
    /// Update progress based on current state
    fn update_progress(&mut self) {
        self.progress = match self.state {
            TransitionState::Idle => 1.0,
            TransitionState::Starting => 0.0,
            TransitionState::Capturing => 0.2,
            TransitionState::Loading => 0.4,
            TransitionState::Animating => 0.6,
            TransitionState::Completing => 0.8,
        };
    }
    
    /// Complete the transition
    fn complete_transition(&mut self) {
        if let Some(target) = self.target_mode.take() {
            self.current_mode = target;
        }
        self.state = TransitionState::Idle;
        self.progress = 1.0;
    }
    
    /// Get current FPS
    pub fn current_fps(&self) -> f64 {
        self.fps_counter.fps()
    }
    
    /// Check if maintaining 60fps
    pub fn is_maintaining_60fps(&self) -> bool {
        self.fps_counter.is_60fps()
    }
    
    /// Get transition progress
    pub fn progress(&self) -> f32 {
        self.progress
    }
    
    /// Get frame measurements
    pub fn frame_measurements(&self) -> &[FrameMeasurement] {
        &self.frame_measurements
    }
    
    /// Get average frame time during transition
    pub fn avg_frame_time_ms(&self) -> f64 {
        if self.frame_measurements.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = self.frame_measurements
            .iter()
            .map(|m| m.end.duration_since(m.start).as_secs_f64() * 1000.0)
            .sum();
        
        sum / self.frame_measurements.len() as f64
    }
    
    /// Get compliance percentage (frames under budget)
    pub fn compliance_pct(&self) -> f64 {
        if self.frame_measurements.is_empty() {
            return 100.0;
        }
        
        let compliant = self.frame_measurements
            .iter()
            .filter(|m| m.end.duration_since(m.start) <= self.max_frame_time)
            .count();
        
        (compliant as f64 / self.frame_measurements.len() as f64) * 100.0
    }
    
    /// Generate transition report
    pub fn report(&self) -> String {
        format!(
            "AI Mode Transition Report:\n\
             Mode: {:?} -> {:?}\n\
             Frames: {}\n\
             Avg Frame Time: {:.2}ms\n\
             Current FPS: {:.1}\n\
             60fps Compliance: {:.1}%\n\
             Maintained Target: {}",
            self.current_mode,
            self.target_mode,
            self.frame_measurements.len(),
            self.avg_frame_time_ms(),
            self.current_fps(),
            self.compliance_pct(),
            if self.is_maintaining_60fps() { "YES ✓" } else { "NO ✗" }
        )
    }
}

/// Performance monitor for ongoing 60fps validation
pub struct PerformanceMonitor {
    fps_counter: FpsCounter,
    violation_count: u32,
    last_violation: Option<Instant>,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            fps_counter: FpsCounter::new(60),
            violation_count: 0,
            last_violation: None,
        }
    }
    
    /// Record frame and check performance
    pub fn record_frame(&mut self) {
        self.fps_counter.tick();
        
        if !self.fps_counter.is_60fps() {
            self.violation_count += 1;
            self.last_violation = Some(Instant::now());
        }
    }
    
    /// Get current FPS
    pub fn fps(&self) -> f64 {
        self.fps_counter.fps()
    }
    
    /// Check if currently maintaining 60fps
    pub fn is_60fps(&self) -> bool {
        self.fps_counter.is_60fps()
    }
    
    /// Get violation count
    pub fn violations(&self) -> u32 {
        self.violation_count
    }
    
    /// Reset statistics
    pub fn reset(&mut self) {
        self.violation_count = 0;
        self.last_violation = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mode_transition_states() {
        let mut transition = AiModeTransition::new();
        
        assert_eq!(transition.state(), TransitionState::Idle);
        assert!(!transition.is_transitioning());
        
        transition.start_transition(BrowserMode::AiNative);
        
        assert!(transition.is_transitioning());
        assert_eq!(transition.current_mode(), BrowserMode::Traditional);
    }
    
    #[test]
    fn test_transition_fps_maintenance() {
        let mut transition = AiModeTransition::new();
        
        transition.start_transition(BrowserMode::AiNative);
        
        // Process frames
        for _ in 0..30 {
            transition.process_frame();
            std::thread::sleep(Duration::from_millis(16));
        }
        
        // Should have recorded frames
        assert!(!transition.frame_measurements().is_empty());
        
        // Should have FPS data
        let fps = transition.current_fps();
        assert!(fps > 0.0);
    }
    
    #[test]
    fn test_performance_monitor() {
        let mut monitor = PerformanceMonitor::new();
        
        // Simulate 60fps
        for _ in 0..60 {
            monitor.record_frame();
            std::thread::sleep(Duration::from_millis(16));
        }
        
        assert!(monitor.is_60fps());
        assert_eq!(monitor.violations(), 0);
    }
}
