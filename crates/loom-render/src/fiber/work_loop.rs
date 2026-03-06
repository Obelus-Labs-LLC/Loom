//! Fiber Work Loop - Interruptible/Resumable Rendering
//!
//! Splits render work into units that can be paused and resumed,
//! enabling concurrent rendering and time-slicing.

use super::scheduler::{FiberScheduler, FrameDeadline, PriorityLane, WorkResult};
use core::time::Duration;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

/// Current state of a fiber
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FiberState {
    /// Work pending, not started
    Pending,
    /// Work in progress
    Working,
    /// Work yielded, will resume
    Yielded,
    /// Work completed
    Completed,
    /// Work was interrupted
    Interrupted,
}

/// A fiber represents a unit of interruptible work
pub struct Fiber {
    /// Unique fiber ID
    pub id: u64,
    /// Current state
    state: Mutex<FiberState>,
    /// Work function that can be resumed
    work_fn: Box<dyn FnMut(&FiberContext) -> FiberResult + Send>,
    /// Progress (0-100)
    progress: AtomicU64,
    /// Whether fiber is currently executing
    is_executing: AtomicBool,
    /// Priority lane
    lane: PriorityLane,
}

impl std::fmt::Debug for Fiber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Fiber")
            .field("id", &self.id)
            .field("state", &*self.state.lock().unwrap())
            .field("progress", &self.progress.load(Ordering::Relaxed))
            .field("lane", &self.lane)
            .finish()
    }
}

/// Context passed to fiber during execution
pub struct FiberContext {
    /// Current frame deadline
    pub deadline: Option<FrameDeadline>,
    /// Time slice allocated for this fiber
    pub time_slice: Duration,
    /// Work start time
    start_time: std::time::Instant,
    /// Whether fiber should yield
    should_yield_flag: AtomicBool,
}

impl FiberContext {
    /// Create new context with time slice
    pub fn with_slice(slice: Duration) -> Self {
        Self {
            deadline: None,
            time_slice,
            start_time: std::time::Instant::now(),
            should_yield_flag: AtomicBool::new(false),
        }
    }

    /// Check if time slice has expired
    pub fn should_yield(&self) -> bool {
        if self.should_yield_flag.load(Ordering::Relaxed) {
            return true;
        }

        self.start_time.elapsed() >= self.time_slice
    }

    /// Time remaining in slice
    pub fn time_remaining(&self) -> Duration {
        let elapsed = self.start_time.elapsed();
        if elapsed >= self.time_slice {
            Duration::ZERO
        } else {
            self.time_slice - elapsed
        }
    }

    /// Request yield
    pub fn request_yield(&self) {
        self.should_yield_flag.store(true, Ordering::Relaxed);
    }
}

/// Result of fiber execution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FiberResult {
    /// Fiber completed all work
    Completed,
    /// Fiber yielded, has more work
    Continue,
    /// Fiber was interrupted
    Interrupted,
    /// Fiber failed
    Failed,
}

/// Work chunk for incremental processing
pub struct WorkChunk {
    /// Total items to process
    pub total: usize,
    /// Items processed so far
    pub processed: usize,
    /// Items to process in this chunk
    pub batch_size: usize,
}

impl WorkChunk {
    /// Create new work chunk
    pub fn new(total: usize, batch_size: usize) -> Self {
        Self {
            total,
            processed: 0,
            batch_size,
        }
    }

    /// Get next batch of work
    pub fn next_batch(&mut self) -> Option<std::ops::Range<usize>> {
        if self.processed >= self.total {
            return None;
        }

        let start = self.processed;
        let end = (start + self.batch_size).min(self.total);
        self.processed = end;

        Some(start..end)
    }

    /// Check if work is complete
    pub fn is_complete(&self) -> bool {
        self.processed >= self.total
    }

    /// Get progress percentage (0-100)
    pub fn progress(&self) -> u64 {
        if self.total == 0 {
            return 100;
        }
        (self.processed as u64 * 100) / self.total as u64
    }
}

impl Fiber {
    /// Create new fiber with work function
    pub fn new<F>(id: u64, lane: PriorityLane, work: F) -> Self
    where
        F: FnMut(&FiberContext) -> FiberResult + Send + 'static,
    {
        Self {
            id,
            state: Mutex::new(FiberState::Pending),
            work_fn: Box::new(work),
            progress: AtomicU64::new(0),
            is_executing: AtomicBool::new(false),
            lane,
        }
    }

    /// Execute fiber with given context
    pub fn execute(&self, context: &FiberContext) -> FiberResult {
        // Check if already executing
        if self.is_executing.swap(true, Ordering::SeqCst) {
            return FiberResult::Interrupted;
        }

        // Update state
        *self.state.lock().unwrap() = FiberState::Working;

        // Execute work
        let result = (self.work_fn)(context);

        // Update state based on result
        let new_state = match result {
            FiberResult::Completed => FiberState::Completed,
            FiberResult::Continue => FiberState::Yielded,
            FiberResult::Interrupted => FiberState::Interrupted,
            FiberResult::Failed => FiberState::Interrupted,
        };
        *self.state.lock().unwrap() = new_state;

        // Update progress if completed
        if result == FiberResult::Completed {
            self.progress.store(100, Ordering::Relaxed);
        }

        self.is_executing.store(false, Ordering::SeqCst);
        result
    }

    /// Get current state
    pub fn state(&self) -> FiberState {
        *self.state.lock().unwrap()
    }

    /// Get progress (0-100)
    pub fn progress(&self) -> u64 {
        self.progress.load(Ordering::Relaxed)
    }

    /// Check if fiber is pending
    pub fn is_pending(&self) -> bool {
        self.state() == FiberState::Pending
    }

    /// Check if fiber is completed
    pub fn is_completed(&self) -> bool {
        self.state() == FiberState::Completed
    }

    /// Check if fiber can be resumed
    pub fn is_resumable(&self) -> bool {
        matches!(self.state(), FiberState::Yielded | FiberState::Interrupted)
    }
}

/// Double-buffered fiber queue for concurrent rendering
pub struct FiberQueue {
    /// Current (active) buffer
    current: Mutex<Vec<Arc<Fiber>>>,
    /// Next (pending) buffer
    next: Mutex<Vec<Arc<Fiber>>>,
    /// Completed fibers from current frame
    completed: Mutex<Vec<Arc<Fiber>>>,
    /// Current fiber ID
    next_id: AtomicU64,
}

impl FiberQueue {
    /// Create new double-buffered queue
    pub fn new() -> Self {
        Self {
            current: Mutex::new(Vec::new()),
            next: Mutex::new(Vec::new()),
            completed: Mutex::new(Vec::new()),
            next_id: AtomicU64::new(1),
        }
    }

    /// Schedule a new fiber
    pub fn schedule<F>(&self, lane: PriorityLane, work: F) -> Arc<Fiber>
    where
        F: FnMut(&FiberContext) -> FiberResult + Send + 'static,
    {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let fiber = Arc::new(Fiber::new(id, lane, work));
        
        // Add to next buffer (will be in next frame)
        self.next.lock().unwrap().push(Arc::clone(&fiber));
        
        fiber
    }

    /// Swap buffers (start new frame)
    pub fn swap_buffers(&self) {
        let mut current = self.current.lock().unwrap();
        let mut next = self.next.lock().unwrap();
        let mut completed = self.completed.lock().unwrap();

        // Move completed from current to completed list
        completed.retain(|f| !f.is_completed());
        for fiber in current.drain(..) {
            if f.is_completed() {
                completed.push(fiber);
            } else if f.is_resumable() {
                // Resumable fibers go to next frame
                next.push(fiber);
            }
            // Abandoned fibers are dropped
        }

        // Swap current and next
        std::mem::swap(&mut *current, &mut *next);
    }

    /// Get fibers in current buffer
    pub fn current_fibers(&self) -> Vec<Arc<Fiber>> {
        self.current.lock().unwrap().clone()
    }

    /// Check if current buffer has work
    pub fn has_work(&self) -> bool {
        !self.current.lock().unwrap().is_empty()
    }

    /// Get count of pending fibers in next buffer
    pub fn pending_count(&self) -> usize {
        self.next.lock().unwrap().len()
    }

    /// Get count of completed fibers
    pub fn completed_count(&self) -> usize {
        self.completed.lock().unwrap().len()
    }

    /// Clear all buffers
    pub fn clear(&self) {
        self.current.lock().unwrap().clear();
        self.next.lock().unwrap().clear();
        self.completed.lock().unwrap().clear();
    }
}

impl Default for FiberQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Main Fiber work loop
pub struct FiberWorkLoop {
    /// Scheduler for time-slicing
    scheduler: Arc<FiberScheduler>,
    /// Fiber queue with double buffering
    queue: Arc<FiberQueue>,
    /// Whether loop is running
    running: AtomicBool,
    /// Work loop statistics
    stats: Mutex<WorkLoopStats>,
}

/// Work loop statistics
#[derive(Debug, Clone, Default)]
pub struct WorkLoopStats {
    /// Total frames processed
    pub total_frames: u64,
    /// Total fibers executed
    pub total_fibers: u64,
    /// Fibers completed this frame
    pub fibers_completed: u64,
    /// Fibers yielded this frame
    pub fibers_yielded: u64,
    /// Average fibers per frame
    pub avg_fibers_per_frame: f64,
}

impl FiberWorkLoop {
    /// Create new work loop
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            scheduler: FiberScheduler::new(),
            queue: Arc::new(FiberQueue::new()),
            running: AtomicBool::new(false),
            stats: Mutex::new(WorkLoopStats::default()),
        })
    }

    /// Schedule fiber work
    pub fn schedule<F>(&self, lane: PriorityLane, work: F) -> Arc<Fiber>
    where
        F: FnMut(&FiberContext) -> FiberResult + Send + 'static,
    {
        self.queue.schedule(lane, work)
    }

    /// Run single frame of work
    pub fn run_frame(&self) -> FrameResult {
        // Swap buffers for new frame
        self.queue.swap_buffers();

        // Begin frame deadline tracking
        let deadline = FrameDeadline::new_60fps();
        
        let fibers = self.queue.current_fibers();
        let mut completed = 0u64;
        let mut yielded = 0u64;

        // Process fibers until deadline or all complete
        for fiber in fibers {
            // Check if we should yield before starting new fiber
            if deadline.should_yield() {
                break;
            }

            // Create context with appropriate time slice
            let context = FiberContext::with_slice(fiber.lane.time_slice());

            // Execute fiber
            let result = fiber.execute(&context);

            match result {
                FiberResult::Completed => completed += 1,
                FiberResult::Continue => yielded += 1,
                FiberResult::Interrupted => yielded += 1,
                FiberResult::Failed => {}
            }

            // Check deadline again
            if deadline.should_yield() {
                break;
            }
        }

        // Update stats
        {
            let mut stats = self.stats.lock().unwrap();
            stats.total_frames += 1;
            stats.total_fibers += completed + yielded;
            stats.fibers_completed = completed;
            stats.fibers_yielded = yielded;
            stats.avg_fibers_per_frame = stats.total_fibers as f64 / stats.total_frames as f64;
        }

        // Complete frame and get metrics
        let metrics = deadline.complete();

        FrameResult {
            fibers_completed: completed,
            fibers_yielded: yielded,
            met_deadline: metrics.met_deadline,
            frame_time: metrics.actual_time,
        }
    }

    /// Run continuous work loop
    pub fn run(self: &Arc<Self>) {
        self.running.store(true, Ordering::SeqCst);

        while self.running.load(Ordering::SeqCst) {
            // Run one frame of work
            let _result = self.run_frame();

            // Post-frame cleanup
            self.post_frame_cleanup();

            // Small yield to prevent busy-waiting
            if !self.queue.has_work() && self.queue.pending_count() == 0 {
                std::thread::sleep(Duration::from_millis(1));
            }
        }
    }

    /// Stop work loop
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    /// Post-frame cleanup
    fn post_frame_cleanup(&self) {
        // Cleanup completed fibers
        // In full implementation, this would:
        // - Free temporary allocations
        // - Update component state
        // - Trigger effects
    }

    /// Get current statistics
    pub fn stats(&self) -> WorkLoopStats {
        self.stats.lock().unwrap().clone()
    }

    /// Check if loop is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Get queue reference
    pub fn queue(&self) -> &FiberQueue {
        &self.queue
    }

    /// Get scheduler reference
    pub fn scheduler(&self) -> &FiberScheduler {
        &self.scheduler
    }
}

/// Result of a frame of work
#[derive(Debug, Clone, Copy)]
pub struct FrameResult {
    /// Fibers completed this frame
    pub fibers_completed: u64,
    /// Fibers yielded (to continue next frame)
    pub fibers_yielded: u64,
    /// Whether frame met deadline
    pub met_deadline: bool,
    /// Total frame time
    pub frame_time: Duration,
}

/// Utility function to create incremental work fiber
pub fn incremental_fiber<F>(
    total_items: usize,
    batch_size: usize,
    mut process_batch: F,
) -> impl FnMut(&FiberContext) -> FiberResult
where
    F: FnMut(std::ops::Range<usize>) + Send,
{
    let mut chunk = WorkChunk::new(total_items, batch_size);

    move |ctx: &FiberContext| {
        while let Some(batch) = chunk.next_batch() {
            // Check if we should yield before processing
            if ctx.should_yield() {
                return FiberResult::Continue;
            }

            // Process this batch
            process_batch(batch);
        }

        FiberResult::Completed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_work_chunk() {
        let mut chunk = WorkChunk::new(100, 10);
        
        assert_eq!(chunk.next_batch(), Some(0..10));
        assert_eq!(chunk.next_batch(), Some(10..20));
        
        // Process remaining
        while chunk.next_batch().is_some() {}
        
        assert!(chunk.is_complete());
        assert_eq!(chunk.progress(), 100);
    }

    #[test]
    fn test_fiber_execution() {
        let fiber = Fiber::new(1, PriorityLane::Normal, |_ctx| {
            FiberResult::Completed
        });

        assert!(fiber.is_pending());

        let ctx = FiberContext::with_slice(Duration::from_millis(5));
        let result = fiber.execute(&ctx);

        assert_eq!(result, FiberResult::Completed);
        assert!(fiber.is_completed());
        assert_eq!(fiber.progress(), 100);
    }

    #[test]
    fn test_fiber_yielding() {
        let fiber = Fiber::new(1, PriorityLane::Normal, |ctx| {
            if ctx.should_yield() {
                FiberResult::Continue
            } else {
                FiberResult::Completed
            }
        });

        let ctx = FiberContext::with_slice(Duration::ZERO);
        let result = fiber.execute(&ctx);

        assert_eq!(result, FiberResult::Continue);
        assert!(fiber.is_resumable());
    }

    #[test]
    fn test_double_buffering() {
        let queue = FiberQueue::new();

        // Schedule fiber
        let fiber = queue.schedule(PriorityLane::Normal, |_ctx| FiberResult::Completed);
        
        // Initially in next buffer
        assert_eq!(queue.pending_count(), 1);
        assert!(!queue.has_work());

        // Swap to current
        queue.swap_buffers();
        
        assert!(queue.has_work());
        assert_eq!(queue.pending_count(), 0);
    }

    #[test]
    fn test_incremental_fiber() {
        let processed = Arc::new(Mutex::new(Vec::new()));
        let processed_clone = Arc::clone(&processed);

        let work = incremental_fiber(100, 10, move |batch| {
            processed_clone.lock().unwrap().push(batch);
        });

        let fiber = Fiber::new(1, PriorityLane::Normal, work);
        
        // Execute with generous time slice (should complete)
        let ctx = FiberContext::with_slice(Duration::from_secs(1));
        let result = fiber.execute(&ctx);

        assert_eq!(result, FiberResult::Completed);
        assert_eq!(processed.lock().unwrap().len(), 10); // 100 items / 10 batch = 10 batches
    }
}
