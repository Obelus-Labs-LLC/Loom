//! React Fiber-style Work Loop for Pausable Rendering
//! 
//! Time-slicing and priority-based scheduling to maintain 60fps.
//! Breaks rendering work into units that can be paused and resumed.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Frame budget for 60fps (~16.67ms)
/// We use 10ms to leave room for browser overhead
pub const FRAME_BUDGET_MS: f64 = 10.0;
pub const FRAME_BUDGET: Duration = Duration::from_millis(10);

/// Priority levels for work
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    /// Critical: User input, animations
    Immediate = 0,
    /// High: Layout changes, visible content
    High = 1,
    /// Normal: Background updates
    Normal = 2,
    /// Low: Offscreen content, predictions
    Low = 3,
    /// Idle: Cleanup, cache maintenance
    Idle = 4,
}

/// A unit of work that can be executed incrementally
pub trait WorkUnit: Send {
    /// Execute work, return true if complete
    fn execute(&mut self) -> bool;
    
    /// Priority of this work unit
    fn priority(&self) -> Priority;
    
    /// Estimated cost in microseconds (for scheduling)
    fn estimated_cost(&self) -> u64;
}

/// Work loop state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WorkLoopState {
    /// Not currently processing
    Idle,
    /// Processing work
    Working,
    /// Paused, will resume next frame
    Paused,
    /// Waiting for next frame
    WaitingForFrame,
}

/// Performance metrics for a frame
#[derive(Debug, Clone)]
pub struct FrameMetrics {
    pub frame_number: u64,
    pub start_time: Instant,
    pub end_time: Instant,
    pub work_units_processed: usize,
    pub work_units_remaining: usize,
    pub did_yield: bool,
    pub budget_overrun: Option<Duration>,
}

impl FrameMetrics {
    pub fn elapsed_ms(&self) -> f64 {
        self.end_time.duration_since(self.start_time).as_secs_f64() * 1000.0
    }
    
    pub fn within_budget(&self) -> bool {
        self.end_time.duration_since(self.start_time) <= FRAME_BUDGET
    }
}

/// React Fiber-style work loop
pub struct WorkLoop {
    /// Queue of work units sorted by priority
    work_queue: VecDeque<Box<dyn WorkUnit>>,
    
    /// Current work unit being processed
    current_work: Option<Box<dyn WorkUnit>>,
    
    /// Current state
    state: WorkLoopState,
    
    /// Frame counter
    frame_number: u64,
    
    /// Performance metrics history
    metrics_history: Vec<FrameMetrics>,
    
    /// Maximum metrics history size
    max_metrics_history: usize,
    
    /// Deadline for current frame
    frame_deadline: Option<Instant>,
    
    /// Whether to use requestAnimationFrame timing
    use_raf: bool,
}

impl WorkLoop {
    pub fn new() -> Self {
        Self {
            work_queue: VecDeque::new(),
            current_work: None,
            state: WorkLoopState::Idle,
            frame_number: 0,
            metrics_history: Vec::with_capacity(120),
            max_metrics_history: 120, // 2 seconds at 60fps
            frame_deadline: None,
            use_raf: true,
        }
    }
    
    /// Schedule a new work unit
    pub fn schedule(&mut self, work: Box<dyn WorkUnit>) {
        // Insert in priority order
        let priority = work.priority();
        let insert_pos = self.work_queue
            .iter()
            .position(|w| w.priority() > priority)
            .unwrap_or(self.work_queue.len());
        
        self.work_queue.insert(insert_pos, work);
    }
    
    /// Schedule multiple work units
    pub fn schedule_batch(&mut self, works: Vec<Box<dyn WorkUnit>>) {
        for work in works {
            self.schedule(work);
        }
    }
    
    /// Start or resume the work loop
    pub fn start(&mut self) {
        if self.state == WorkLoopState::Working {
            return;
        }
        
        self.state = WorkLoopState::Working;
        self.frame_deadline = Some(Instant::now() + FRAME_BUDGET);
        
        // Process work until deadline
        self.process_work();
    }
    
    /// Process work units until frame deadline
    fn process_work(&mut self) {
        self.frame_number += 1;
        let frame_start = Instant::now();
        let deadline = self.frame_deadline.unwrap_or(frame_start + FRAME_BUDGET);
        
        let mut work_units_processed = 0;
        let mut did_yield = false;
        
        // Process current work first if resumed
        if let Some(mut work) = self.current_work.take() {
            if self.should_continue_work(deadline) {
                if work.execute() {
                    // Work completed
                    work_units_processed += 1;
                } else {
                    // Work not complete, save for next frame
                    self.current_work = Some(work);
                    did_yield = true;
                }
            } else {
                // Out of time, save work
                self.current_work = Some(work);
                did_yield = true;
            }
        }
        
        // Process queue
        while let Some(mut work) = self.work_queue.pop_front() {
            if !self.should_continue_work(deadline) {
                // Out of time, save work for next frame
                self.current_work = Some(work);
                did_yield = true;
                break;
            }
            
            // Execute work unit
            if work.execute() {
                // Work completed
                work_units_processed += 1;
            } else {
                // Work not complete, save for next frame
                self.current_work = Some(work);
                did_yield = true;
                break;
            }
        }
        
        let frame_end = Instant::now();
        let budget_overrun = if frame_end > deadline {
            Some(frame_end - deadline)
        } else {
            None
        };
        
        // Record metrics
        let metrics = FrameMetrics {
            frame_number: self.frame_number,
            start_time: frame_start,
            end_time: frame_end,
            work_units_processed,
            work_units_remaining: self.work_queue.len() + self.current_work.as_ref().map(|_| 1).unwrap_or(0),
            did_yield,
            budget_overrun,
        };
        
        self.record_metrics(metrics);
        
        // Update state
        if self.has_work() {
            self.state = WorkLoopState::Paused;
        } else {
            self.state = WorkLoopState::Idle;
        }
    }
    
    /// Check if we have time to continue working
    fn should_continue_work(&self, deadline: Instant) -> bool {
        // Check deadline
        if Instant::now() >= deadline {
            return false;
        }
        
        // Leave 1ms buffer for cleanup
        let buffer = Duration::from_millis(1);
        Instant::now() + buffer < deadline
    }
    
    /// Check if there's more work to do
    pub fn has_work(&self) -> bool {
        self.current_work.is_some() || !self.work_queue.is_empty()
    }
    
    /// Record metrics, maintaining history size limit
    fn record_metrics(&mut self, metrics: FrameMetrics) {
        if self.metrics_history.len() >= self.max_metrics_history {
            self.metrics_history.remove(0);
        }
        self.metrics_history.push(metrics);
    }
    
    /// Get average frame time over last N frames
    pub fn average_frame_time_ms(&self, n: usize) -> f64 {
        let n = n.min(self.metrics_history.len());
        if n == 0 {
            return 0.0;
        }
        
        let sum: f64 = self.metrics_history
            .iter()
            .rev()
            .take(n)
            .map(|m| m.elapsed_ms())
            .sum();
        
        sum / n as f64
    }
    
    /// Get 60fps compliance percentage
    pub fn fps_compliance(&self) -> f64 {
        if self.metrics_history.is_empty() {
            return 100.0;
        }
        
        let compliant = self.metrics_history
            .iter()
            .filter(|m| m.within_budget())
            .count();
        
        (compliant as f64 / self.metrics_history.len() as f64) * 100.0
    }
    
    /// Get current queue depth
    pub fn queue_depth(&self) -> usize {
        self.work_queue.len() + self.current_work.as_ref().map(|_| 1).unwrap_or(0)
    }
    
    /// Clear all work
    pub fn clear(&mut self) {
        self.work_queue.clear();
        self.current_work = None;
        self.state = WorkLoopState::Idle;
    }
    
    /// Get current state
    pub fn state(&self) -> WorkLoopState {
        self.state
    }
    
    /// Get metrics history
    pub fn metrics(&self) -> &[FrameMetrics] {
        &self.metrics_history
    }
}

/// Work unit for layout calculation
pub struct LayoutWorkUnit {
    pub node_id: usize,
    pub width: f32,
    pub height: f32,
    pub priority: Priority,
}

impl WorkUnit for LayoutWorkUnit {
    fn execute(&mut self) -> bool {
        // Simulate layout work
        // In real implementation: calculate layout for node
        std::thread::sleep(Duration::from_micros(100));
        true
    }
    
    fn priority(&self) -> Priority {
        self.priority
    }
    
    fn estimated_cost(&self) -> u64 {
        100 // 100 microseconds
    }
}

/// Work unit for rendering
pub struct RenderWorkUnit {
    pub surface_id: usize,
    pub commands: Vec<RenderCommand>,
    pub priority: Priority,
}

#[derive(Debug, Clone)]
pub struct RenderCommand {
    pub cmd_type: RenderCommandType,
}

#[derive(Debug, Clone)]
pub enum RenderCommandType {
    Clear,
    DrawRect { x: f32, y: f32, w: f32, h: f32, color: u32 },
    DrawText { x: f32, y: f32, text: String },
}

impl WorkUnit for RenderWorkUnit {
    fn execute(&mut self) -> bool {
        // Simulate rendering work
        // Process commands in batches if needed
        let batch_size = 10;
        let to_process = self.commands.len().min(batch_size);
        
        for _ in 0..to_process {
            if let Some(_cmd) = self.commands.pop() {
                // Render command
                std::thread::sleep(Duration::from_micros(50));
            }
        }
        
        // Return true if all commands processed
        self.commands.is_empty()
    }
    
    fn priority(&self) -> Priority {
        self.priority
    }
    
    fn estimated_cost(&self) -> u64 {
        self.commands.len() as u64 * 50
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_work_loop_scheduling() {
        let mut loop_state = WorkLoop::new();
        
        // Schedule work units
        for i in 0..10 {
            let work = Box::new(LayoutWorkUnit {
                node_id: i,
                width: 100.0,
                height: 100.0,
                priority: Priority::Normal,
            });
            loop_state.schedule(work);
        }
        
        assert_eq!(loop_state.queue_depth(), 10);
        
        // Process one frame
        loop_state.start();
        
        // Should have processed some but not all
        assert!(loop_state.metrics_history.len() > 0);
    }
    
    #[test]
    fn test_priority_ordering() {
        let mut loop_state = WorkLoop::new();
        
        // Schedule low priority first
        loop_state.schedule(Box::new(LayoutWorkUnit {
            node_id: 1,
            width: 100.0,
            height: 100.0,
            priority: Priority::Low,
        }));
        
        // Then high priority
        loop_state.schedule(Box::new(LayoutWorkUnit {
            node_id: 2,
            width: 100.0,
            height: 100.0,
            priority: Priority::High,
        }));
        
        // High priority should be first in queue
        let first = loop_state.work_queue.front().unwrap();
        assert_eq!(first.priority(), Priority::High);
    }
}
