//! Render Pipeline Profiler
//!
//! Per-stage timing for layout, paint, and composite phases.
//! Memory allocation tracking for render operations.

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Render pipeline stages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderStage {
    /// Style computation and CSS matching
    Style,
    /// Layout calculation (box model)
    Layout,
    /// Paint preparation (draw calls)
    Paint,
    /// Rasterization (GPU command generation)
    Raster,
    /// Compositing (layer merging)
    Composite,
    /// GPU execution (actual rendering)
    GpuExecute,
    /// Present to screen
    Present,
}

impl RenderStage {
    /// Get all stages in pipeline order
    pub fn all() -> &'static [RenderStage] {
        &[
            RenderStage::Style,
            RenderStage::Layout,
            RenderStage::Paint,
            RenderStage::Raster,
            RenderStage::Composite,
            RenderStage::GpuExecute,
            RenderStage::Present,
        ]
    }

    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            RenderStage::Style => "Style",
            RenderStage::Layout => "Layout",
            RenderStage::Paint => "Paint",
            RenderStage::Raster => "Raster",
            RenderStage::Composite => "Composite",
            RenderStage::GpuExecute => "GPU",
            RenderStage::Present => "Present",
        }
    }
}

/// Timing for a single stage
#[derive(Debug, Clone, Copy, Default)]
pub struct StageTiming {
    /// Time spent in this stage
    pub duration: Duration,
    /// Number of elements processed
    pub element_count: usize,
    /// Whether this stage was skipped (cached)
    pub skipped: bool,
}

/// Memory allocation record
#[derive(Debug, Clone)]
pub struct AllocationRecord {
    /// Allocation size in bytes
    pub size: usize,
    /// Allocation type/category
    pub category: &'static str,
    /// Timestamp
    pub timestamp: Instant,
    /// Stack trace or location
    pub location: &'static str,
}

/// Per-frame render profile
#[derive(Debug, Clone)]
pub struct FrameProfile {
    /// Frame number
    pub frame_num: u64,
    /// Stage timings
    pub stages: HashMap<RenderStage, StageTiming>,
    /// Total frame time
    pub total_time: Duration,
    /// Memory allocations during this frame
    pub allocations: Vec<AllocationRecord>,
    /// Total memory allocated this frame
    pub total_allocated: usize,
    /// Timestamp
    pub timestamp: Instant,
}

impl FrameProfile {
    /// Create new empty profile
    pub fn new(frame_num: u64) -> Self {
        Self {
            frame_num,
            stages: HashMap::new(),
            total_time: Duration::ZERO,
            allocations: Vec::new(),
            total_allocated: 0,
            timestamp: Instant::now(),
        }
    }

    /// Record stage timing
    pub fn record_stage(&mut self, stage: RenderStage, timing: StageTiming) {
        self.stages.insert(stage, timing);
    }

    /// Record memory allocation
    pub fn record_allocation(&mut self, size: usize, category: &'static str, location: &'static str) {
        self.allocations.push(AllocationRecord {
            size,
            category,
            timestamp: Instant::now(),
            location,
        });
        self.total_allocated += size;
    }

    /// Get time for specific stage
    pub fn stage_time(&self, stage: RenderStage) -> Duration {
        self.stages.get(&stage).map(|t| t.duration).unwrap_or_default()
    }

    /// Get sum of all stage times
    pub fn stages_total(&self) -> Duration {
        self.stages.values().map(|t| t.duration).sum()
    }

    /// Get overhead (time not accounted for in stages)
    pub fn overhead(&self) -> Duration {
        self.total_time.saturating_sub(self.stages_total())
    }

    /// Check if all stages are recorded
    pub fn is_complete(&self) -> bool {
        RenderStage::all().iter().all(|s| self.stages.contains_key(s))
    }
}

/// Render pipeline profiler
pub struct RenderProfiler {
    /// Current frame profile being built
    current: Option<FrameProfile>,
    /// History of completed frame profiles
    history: Vec<FrameProfile>,
    /// Maximum history size
    max_history: usize,
    /// Current frame number
    frame_num: u64,
    /// Stage timers (active)
    stage_timers: HashMap<RenderStage, Instant>,
    /// Memory tracking enabled
    track_memory: bool,
    /// Cumulative allocations by category
    allocation_stats: HashMap<&'static str, usize>,
}

impl RenderProfiler {
    /// Default history size
    pub const DEFAULT_HISTORY: usize = 300; // 5 seconds at 60fps

    /// Create new profiler
    pub fn new() -> Self {
        Self::with_memory_tracking(true)
    }

    /// Create with optional memory tracking
    pub fn with_memory_tracking(track_memory: bool) -> Self {
        Self {
            current: None,
            history: Vec::with_capacity(Self::DEFAULT_HISTORY),
            max_history: Self::DEFAULT_HISTORY,
            frame_num: 0,
            stage_timers: HashMap::new(),
            track_memory,
            allocation_stats: HashMap::new(),
        }
    }

    /// Begin profiling a new frame
    pub fn begin_frame(&mut self) {
        // Complete previous frame if exists
        if self.current.is_some() {
            self.end_frame();
        }

        self.frame_num += 1;
        self.current = Some(FrameProfile::new(self.frame_num));
        self.stage_timers.clear();
    }

    /// Start timing a stage
    pub fn start_stage(&mut self, stage: RenderStage) {
        self.stage_timers.insert(stage, Instant::now());
    }

    /// End timing a stage
    pub fn end_stage(&mut self, stage: RenderStage, element_count: usize) {
        if let Some(start) = self.stage_timers.remove(&stage) {
            let duration = start.elapsed();
            
            if let Some(ref mut profile) = self.current {
                profile.record_stage(stage, StageTiming {
                    duration,
                    element_count,
                    skipped: false,
                });
            }
        }
    }

    /// Mark stage as skipped (cached)
    pub fn skip_stage(&mut self, stage: RenderStage) {
        if let Some(ref mut profile) = self.current {
            profile.record_stage(stage, StageTiming {
                duration: Duration::ZERO,
                element_count: 0,
                skipped: true,
            });
        }
    }

    /// Record memory allocation
    pub fn record_alloc(&mut self, size: usize, category: &'static str, location: &'static str) {
        if !self.track_memory {
            return;
        }

        if let Some(ref mut profile) = self.current {
            profile.record_allocation(size, category, location);
        }

        // Update cumulative stats
        *self.allocation_stats.entry(category).or_insert(0) += size;
    }

    /// End current frame and store profile
    pub fn end_frame(&mut self) -> Option<FrameProfile> {
        let mut profile = self.current.take()?;
        profile.total_time = profile.timestamp.elapsed();

        // Store in history
        if self.history.len() >= self.max_history {
            self.history.remove(0);
        }
        
        let result = profile.clone();
        self.history.push(profile);
        
        Some(result)
    }

    /// Get average time for a stage over last N frames
    pub fn average_stage_time(&self, stage: RenderStage, n: usize) -> Duration {
        let n = n.min(self.history.len());
        if n == 0 {
            return Duration::ZERO;
        }

        let total: Duration = self.history.iter().rev().take(n)
            .filter_map(|p| p.stages.get(&stage).map(|t| t.duration))
            .sum();

        total / n as u32
    }

    /// Get bottleneck stage (slowest on average)
    pub fn bottleneck_stage(&self, n: usize) -> Option<(RenderStage, Duration)> {
        let mut max_time = Duration::ZERO;
        let mut bottleneck = None;

        for stage in RenderStage::all() {
            let avg = self.average_stage_time(*stage, n);
            if avg > max_time {
                max_time = avg;
                bottleneck = Some((*stage, avg));
            }
        }

        bottleneck
    }

    /// Get average frame time
    pub fn average_frame_time(&self, n: usize) -> Duration {
        let n = n.min(self.history.len());
        if n == 0 {
            return Duration::ZERO;
        }

        let total: Duration = self.history.iter().rev().take(n)
            .map(|p| p.total_time)
            .sum();

        total / n as u32
    }

    /// Get average overhead (unaccounted time)
    pub fn average_overhead(&self, n: usize) -> Duration {
        let n = n.min(self.history.len());
        if n == 0 {
            return Duration::ZERO;
        }

        let total: Duration = self.history.iter().rev().take(n)
            .map(|p| p.overhead())
            .sum();

        total / n as u32
    }

    /// Get total memory allocated in last N frames
    pub fn total_allocations(&self, n: usize) -> usize {
        self.history.iter().rev().take(n)
            .map(|p| p.total_allocated)
            .sum()
    }

    /// Get allocation stats by category
    pub fn allocation_stats(&self) -> &HashMap<&'static str, usize> {
        &self.allocation_stats
    }

    /// Generate report for last N frames
    pub fn generate_report(&self, n: usize) -> RenderReport {
        let n = n.min(self.history.len());
        
        RenderReport {
            frame_count: n,
            avg_frame_time: self.average_frame_time(n),
            avg_overhead: self.average_overhead(n),
            stage_averages: RenderStage::all().iter()
                .map(|&s| (s, self.average_stage_time(s, n)))
                .collect(),
            bottleneck: self.bottleneck_stage(n),
            total_allocations: self.total_allocations(n),
            allocation_breakdown: self.allocation_stats.clone(),
        }
    }

    /// Reset all profiling data
    pub fn reset(&mut self) {
        self.current = None;
        self.history.clear();
        self.frame_num = 0;
        self.stage_timers.clear();
        self.allocation_stats.clear();
    }

    /// Get current frame number
    pub fn current_frame(&self) -> u64 {
        self.frame_num
    }

    /// Get history size
    pub fn history_len(&self) -> usize {
        self.history.len()
    }
}

impl Default for RenderProfiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Render performance report
#[derive(Debug, Clone)]
pub struct RenderReport {
    /// Number of frames analyzed
    pub frame_count: usize,
    /// Average frame time
    pub avg_frame_time: Duration,
    /// Average unaccounted overhead
    pub avg_overhead: Duration,
    /// Average time per stage
    pub stage_averages: Vec<(RenderStage, Duration)>,
    /// Current bottleneck stage
    pub bottleneck: Option<(RenderStage, Duration)>,
    /// Total memory allocated
    pub total_allocations: usize,
    /// Allocations by category
    pub allocation_breakdown: HashMap<&'static str, usize>,
}

impl RenderReport {
    /// Format as human-readable string
    pub fn format(&self) -> String {
        let mut output = String::new();
        
        output.push_str("=== Render Performance Report ===\n");
        output.push_str(&format!("Frames analyzed: {}\n", self.frame_count));
        output.push_str(&format!("Avg frame time: {:?}\n", self.avg_frame_time));
        output.push_str(&format!("Avg overhead:   {:?}\n", self.avg_overhead));
        output.push_str(&format!("Effective FPS:  {:.1}\n", 
            1.0 / self.avg_frame_time.as_secs_f64()));
        
        output.push_str("\n--- Stage Breakdown ---\n");
        for (stage, time) in &self.stage_averages {
            let pct = if !self.avg_frame_time.is_zero() {
                time.as_nanos() as f64 / self.avg_frame_time.as_nanos() as f64 * 100.0
            } else {
                0.0
            };
            output.push_str(&format!("  {:12} {:>10?} ({:5.1}%)\n", 
                stage.name(), time, pct));
        }
        
        if let Some((bottleneck, time)) = self.bottleneck {
            output.push_str(&format!("\n⚠ BOTTLENECK: {} ({:?})\n", 
                bottleneck.name(), time));
        }
        
        if self.total_allocations > 0 {
            output.push_str(&format!("\n--- Memory Allocations ---\n"));
            output.push_str(&format!("Total: {} bytes ({:.2} MB)\n", 
                self.total_allocations,
                self.total_allocations as f64 / (1024.0 * 1024.0)));
            
            let mut categories: Vec<_> = self.allocation_breakdown.iter().collect();
            categories.sort_by(|a, b| b.1.cmp(a.1));
            
            for (cat, size) in categories.iter().take(5) {
                output.push_str(&format!("  {:20} {:>10} bytes\n", cat, size));
            }
        }
        
        output
    }
}

/// Scoped stage timer for RAII profiling
pub struct StageTimer<'a> {
    profiler: &'a mut RenderProfiler,
    stage: RenderStage,
    element_count: usize,
}

impl<'a> StageTimer<'a> {
    /// Start timing a stage
    pub fn new(profiler: &'a mut RenderProfiler, stage: RenderStage) -> Self {
        profiler.start_stage(stage);
        Self {
            profiler,
            stage,
            element_count: 0,
        }
    }

    /// Set element count for this stage
    pub fn with_count(mut self, count: usize) -> Self {
        self.element_count = count;
        self
    }
}

impl<'a> Drop for StageTimer<'a> {
    fn drop(&mut self) {
        self.profiler.end_stage(self.stage, self.element_count);
    }
}

/// Scoped frame timer for RAII profiling
pub struct ProfiledFrame<'a> {
    profiler: &'a mut RenderProfiler,
}

impl<'a> ProfiledFrame<'a> {
    /// Start profiling a frame
    pub fn new(profiler: &'a mut RenderProfiler) -> Self {
        profiler.begin_frame();
        Self { profiler }
    }

    /// Start timing a stage within this frame
    pub fn stage(&mut self, stage: RenderStage) -> StageTimer<'_> {
        StageTimer::new(self.profiler, stage)
    }

    /// Record allocation
    pub fn alloc(&mut self, size: usize, category: &'static str, location: &'static str) {
        self.profiler.record_alloc(size, category, location);
    }
}

impl<'a> Drop for ProfiledFrame<'a> {
    fn drop(&mut self) {
        self.profiler.end_frame();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profiler_basic() {
        let mut profiler = RenderProfiler::new();
        
        // Profile a frame
        profiler.begin_frame();
        
        profiler.start_stage(RenderStage::Layout);
        std::thread::sleep(Duration::from_millis(5));
        profiler.end_stage(RenderStage::Layout, 100);
        
        profiler.start_stage(RenderStage::Paint);
        std::thread::sleep(Duration::from_millis(3));
        profiler.end_stage(RenderStage::Paint, 50);
        
        let profile = profiler.end_frame().unwrap();
        
        assert_eq!(profile.frame_num, 1);
        assert!(profile.stage_time(RenderStage::Layout) >= Duration::from_millis(5));
        assert!(profile.stage_time(RenderStage::Paint) >= Duration::from_millis(3));
    }

    #[test]
    fn test_bottleneck_detection() {
        let mut profiler = RenderProfiler::new();
        
        // Create 10 frames with layout being slowest
        for _ in 0..10 {
            profiler.begin_frame();
            
            profiler.start_stage(RenderStage::Style);
            profiler.end_stage(RenderStage::Style, 10);
            
            profiler.start_stage(RenderStage::Layout);
            std::thread::sleep(Duration::from_millis(10));
            profiler.end_stage(RenderStage::Layout, 100);
            
            profiler.start_stage(RenderStage::Paint);
            profiler.end_stage(RenderStage::Paint, 50);
            
            profiler.end_frame();
        }
        
        let bottleneck = profiler.bottleneck_stage(10);
        assert!(bottleneck.is_some());
        assert_eq!(bottleneck.unwrap().0, RenderStage::Layout);
    }

    #[test]
    fn test_memory_tracking() {
        let mut profiler = RenderProfiler::with_memory_tracking(true);
        
        profiler.begin_frame();
        profiler.record_alloc(1024, "nodes", "layout.rs:42");
        profiler.record_alloc(2048, "textures", "paint.rs:10");
        profiler.record_alloc(512, "nodes", "layout.rs:55");
        
        let profile = profiler.end_frame().unwrap();
        
        assert_eq!(profile.total_allocated, 1024 + 2048 + 512);
        assert_eq!(profile.allocations.len(), 3);
        
        let stats = profiler.allocation_stats();
        assert_eq!(stats.get("nodes"), Some(&1536usize));
        assert_eq!(stats.get("textures"), Some(&2048usize));
    }

    #[test]
    fn test_report_generation() {
        let mut profiler = RenderProfiler::new();
        
        for _ in 0..5 {
            profiler.begin_frame();
            profiler.start_stage(RenderStage::Layout);
            std::thread::sleep(Duration::from_millis(2));
            profiler.end_stage(RenderStage::Layout, 10);
            profiler.end_frame();
        }
        
        let report = profiler.generate_report(5);
        assert_eq!(report.frame_count, 5);
        assert!(report.avg_frame_time >= Duration::from_millis(2));
        
        let output = report.format();
        assert!(output.contains("Render Performance Report"));
        assert!(output.contains("Layout"));
    }
}
