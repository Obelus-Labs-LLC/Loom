//! FPS Counter and Frame Time Histogram
//!
//! Provides high-precision frame timing with statistical analysis.
//! Tracks GPU vs CPU time split for render pipeline optimization.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Frame time statistics (p50, p95, p99)
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct FrameStats {
    /// 50th percentile (median)
    pub p50: Duration,
    /// 95th percentile
    pub p95: Duration,
    /// 99th percentile
    pub p99: Duration,
    /// Mean frame time
    pub mean: Duration,
    /// Minimum frame time
    pub min: Duration,
    /// Maximum frame time
    pub max: Duration,
    /// Standard deviation
    pub std_dev: Duration,
}

/// GPU vs CPU time split
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct SplitTime {
    /// CPU time (layout, paint preparation)
    pub cpu: Duration,
    /// GPU time (rendering, composition)
    pub gpu: Duration,
    /// Total frame time
    pub total: Duration,
}

/// Single frame measurement
#[derive(Debug, Clone, Copy)]
pub struct FrameMeasurement {
    /// Frame number
    pub frame_num: u64,
    /// Total frame time
    pub total_time: Duration,
    /// CPU time
    pub cpu_time: Duration,
    /// GPU time
    pub gpu_time: Duration,
    /// Timestamp when frame started
    pub timestamp: Instant,
}

/// FPS Counter with histogram tracking
pub struct FpsCounter {
    /// Ring buffer of recent frame times (for rolling FPS)
    recent_frames: VecDeque<Duration>,
    /// Full history for statistical analysis
    history: Vec<FrameMeasurement>,
    /// Maximum history size
    max_history: usize,
    /// Current frame number
    frame_num: u64,
    /// Frame start time
    frame_start: Option<Instant>,
    /// CPU measurement start
    cpu_start: Option<Instant>,
    /// GPU measurement start
    gpu_start: Option<Instant>,
    /// Last CPU time
    last_cpu_time: Duration,
    /// Last GPU time
    last_gpu_time: Duration,
    /// Target frame time (for 60fps = 16.67ms)
    target_frame_time: Duration,
}

impl FpsCounter {
    /// Target frame time for 60 FPS
    pub const TARGET_60FPS: Duration = Duration::from_nanos(16_666_667);
    /// Target frame time for 120 FPS
    pub const TARGET_120FPS: Duration = Duration::from_nanos(8_333_333);
    /// Default rolling window size
    pub const DEFAULT_WINDOW: usize = 120;
    /// Default history size
    pub const DEFAULT_HISTORY: usize = 1000;

    /// Create new FPS counter targeting 60 FPS
    pub fn new() -> Self {
        Self::with_target(Self::TARGET_60FPS)
    }

    /// Create with custom target frame time
    pub fn with_target(target: Duration) -> Self {
        Self {
            recent_frames: VecDeque::with_capacity(Self::DEFAULT_WINDOW),
            history: Vec::with_capacity(Self::DEFAULT_HISTORY),
            max_history: Self::DEFAULT_HISTORY,
            frame_num: 0,
            frame_start: None,
            cpu_start: None,
            gpu_start: None,
            last_cpu_time: Duration::ZERO,
            last_gpu_time: Duration::ZERO,
            target_frame_time: target,
        }
    }

    /// Start measuring a new frame
    pub fn begin_frame(&mut self) {
        let now = Instant::now();
        
        // Complete previous frame if exists
        if let Some(start) = self.frame_start {
            let total_time = now - start;
            self.record_frame(total_time);
        }
        
        self.frame_num += 1;
        self.frame_start = Some(now);
        self.cpu_start = Some(now);
        self.gpu_start = None;
    }

    /// Mark CPU work complete, GPU work beginning
    pub fn begin_gpu(&mut self) {
        let now = Instant::now();
        
        if let Some(cpu_start) = self.cpu_start {
            self.last_cpu_time = now - cpu_start;
        }
        
        self.gpu_start = Some(now);
    }

    /// Mark frame complete
    pub fn end_frame(&mut self) -> FrameMeasurement {
        let now = Instant::now();
        
        // Calculate GPU time
        if let Some(gpu_start) = self.gpu_start {
            self.last_gpu_time = now - gpu_start;
        }
        
        // Calculate total time
        let total_time = if let Some(start) = self.frame_start {
            now - start
        } else {
            Duration::ZERO
        };
        
        let measurement = FrameMeasurement {
            frame_num: self.frame_num,
            total_time,
            cpu_time: self.last_cpu_time,
            gpu_time: self.last_gpu_time,
            timestamp: now,
        };
        
        // Store in history
        if self.history.len() >= self.max_history {
            self.history.remove(0);
        }
        self.history.push(measurement);
        
        measurement
    }

    /// Record a completed frame time
    fn record_frame(&mut self, duration: Duration) {
        // Add to recent frames (rolling window)
        if self.recent_frames.len() >= Self::DEFAULT_WINDOW {
            self.recent_frames.pop_front();
        }
        self.recent_frames.push_back(duration);
    }

    /// Get current FPS (rolling average)
    pub fn current_fps(&self) -> f64 {
        if self.recent_frames.is_empty() {
            return 0.0;
        }
        
        let avg_time = self.recent_frames.iter().sum::<Duration>() 
            / self.recent_frames.len() as u32;
        
        if avg_time.is_zero() {
            return 0.0;
        }
        
        1.0 / avg_time.as_secs_f64()
    }

    /// Get rolling FPS over last N frames
    pub fn rolling_fps(&self, n: usize) -> f64 {
        let n = n.min(self.recent_frames.len());
        if n == 0 {
            return 0.0;
        }
        
        let recent: Vec<_> = self.recent_frames.iter().rev().take(n).collect();
        let avg_time: Duration = recent.iter().copied().sum::<Duration>() / n as u32;
        
        if avg_time.is_zero() {
            return 0.0;
        }
        
        1.0 / avg_time.as_secs_f64()
    }

    /// Calculate frame time statistics
    pub fn frame_stats(&self) -> FrameStats {
        if self.history.is_empty() {
            return FrameStats::default();
        }
        
        let mut times: Vec<Duration> = self.history.iter()
            .map(|m| m.total_time)
            .collect();
        
        times.sort();
        
        let n = times.len();
        let sum: Duration = times.iter().sum();
        let mean = sum / n as u32;
        
        // Calculate standard deviation
        let variance: f64 = times.iter()
            .map(|&t| {
                let diff = t.as_nanos() as f64 - mean.as_nanos() as f64;
                diff * diff
            })
            .sum::<f64>() / n as f64;
        
        let std_dev_ns = variance.sqrt() as u64;
        let std_dev = Duration::from_nanos(std_dev_ns);
        
        FrameStats {
            p50: times[n / 2],
            p95: times[n * 95 / 100],
            p99: times[n * 99 / 100],
            mean,
            min: times[0],
            max: times[n - 1],
            std_dev,
        }
    }

    /// Get GPU vs CPU split for last frame
    pub fn last_split(&self) -> SplitTime {
        SplitTime {
            cpu: self.last_cpu_time,
            gpu: self.last_gpu_time,
            total: self.last_cpu_time + self.last_gpu_time,
        }
    }

    /// Get average GPU vs CPU split over all history
    pub fn average_split(&self) -> SplitTime {
        if self.history.is_empty() {
            return SplitTime::default();
        }
        
        let total_cpu: Duration = self.history.iter().map(|m| m.cpu_time).sum();
        let total_gpu: Duration = self.history.iter().map(|m| m.gpu_time).sum();
        let n = self.history.len() as u32;
        
        SplitTime {
            cpu: total_cpu / n,
            gpu: total_gpu / n,
            total: (total_cpu + total_gpu) / n,
        }
    }

    /// Get frames that missed target (frame time > target)
    pub fn missed_frames(&self) -> Vec<&FrameMeasurement> {
        self.history.iter()
            .filter(|m| m.total_time > self.target_frame_time)
            .collect()
    }

    /// Get percentage of frames that hit target
    pub fn target_hit_rate(&self) -> f64 {
        if self.history.is_empty() {
            return 0.0;
        }
        
        let hits = self.history.iter()
            .filter(|m| m.total_time <= self.target_frame_time)
            .count();
        
        hits as f64 / self.history.len() as f64 * 100.0
    }

    /// Check if currently maintaining target FPS
    pub fn is_maintaining_target(&self) -> bool {
        self.current_fps() >= (1.0 / self.target_frame_time.as_secs_f64()) * 0.95
    }

    /// Reset all measurements
    pub fn reset(&mut self) {
        self.recent_frames.clear();
        self.history.clear();
        self.frame_num = 0;
        self.frame_start = None;
        self.cpu_start = None;
        self.gpu_start = None;
        self.last_cpu_time = Duration::ZERO;
        self.last_gpu_time = Duration::ZERO;
    }

    /// Get total frames recorded
    pub fn total_frames(&self) -> usize {
        self.history.len()
    }

    /// Generate histogram buckets for frame times
    pub fn histogram(&self, buckets: usize) -> Vec<(Duration, usize)> {
        if self.history.is_empty() || buckets == 0 {
            return Vec::new();
        }
        
        let stats = self.frame_stats();
        let min_ns = stats.min.as_nanos() as f64;
        let max_ns = stats.max.as_nanos() as f64;
        let range = max_ns - min_ns;
        
        if range == 0.0 {
            return vec![(stats.min, self.history.len())];
        }
        
        let bucket_size = range / buckets as f64;
        let mut counts = vec![0usize; buckets];
        
        for m in &self.history {
            let ns = m.total_time.as_nanos() as f64;
            let bucket = ((ns - min_ns) / bucket_size) as usize;
            let bucket = bucket.min(buckets - 1);
            counts[bucket] += 1;
        }
        
        counts.iter().enumerate()
            .map(|(i, &count)| {
                let start_ns = min_ns + (i as f64 * bucket_size);
                (Duration::from_nanos(start_ns as u64), count)
            })
            .collect()
    }
}

impl Default for FpsCounter {
    fn default() -> Self {
        Self::new()
    }
}

/// Scoped frame timer for RAII-style measurement
pub struct FrameTimer<'a> {
    counter: &'a mut FpsCounter,
}

impl<'a> FrameTimer<'a> {
    /// Start a new frame timing scope
    pub fn new(counter: &'a mut FpsCounter) -> Self {
        counter.begin_frame();
        Self { counter }
    }

    /// Mark transition to GPU
    pub fn begin_gpu(&mut self) {
        self.counter.begin_gpu();
    }
}

impl<'a> Drop for FrameTimer<'a> {
    fn drop(&mut self) {
        self.counter.end_frame();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fps_counter_basic() {
        let mut counter = FpsCounter::new();
        
        // Simulate 60 FPS
        for _ in 0..60 {
            counter.begin_frame();
            std::thread::sleep(Duration::from_millis(16));
            counter.end_frame();
        }
        
        assert!(counter.current_fps() > 30.0); // Should be around 60
        assert_eq!(counter.total_frames(), 60);
    }

    #[test]
    fn test_frame_stats() {
        let mut counter = FpsCounter::new();
        
        // Add some frames with varying times
        for i in 0..100 {
            counter.begin_frame();
            counter.last_cpu_time = Duration::from_millis(10 + (i % 10) as u64);
            counter.last_gpu_time = Duration::from_millis(5);
            counter.end_frame();
        }
        
        let stats = counter.frame_stats();
        assert!(!stats.mean.is_zero());
        assert!(stats.max >= stats.min);
        assert!(stats.p95 >= stats.p50);
        assert!(stats.p99 >= stats.p95);
    }

    #[test]
    fn test_split_time() {
        let mut counter = FpsCounter::new();
        
        counter.begin_frame();
        std::thread::sleep(Duration::from_millis(5));
        counter.begin_gpu();
        std::thread::sleep(Duration::from_millis(5));
        let split = counter.end_frame();
        
        assert!(split.cpu_time >= Duration::from_millis(5));
        assert!(split.gpu_time >= Duration::from_millis(5));
    }

    #[test]
    fn test_target_hit_rate() {
        let mut counter = FpsCounter::with_target(Duration::from_millis(20));
        
        // 50 frames at 16ms (hit), 50 at 30ms (miss)
        for i in 0..100 {
            counter.begin_frame();
            if i < 50 {
                counter.last_cpu_time = Duration::from_millis(10);
                counter.last_gpu_time = Duration::from_millis(6);
            } else {
                counter.last_cpu_time = Duration::from_millis(20);
                counter.last_gpu_time = Duration::from_millis(10);
            }
            counter.end_frame();
        }
        
        let hit_rate = counter.target_hit_rate();
        assert!(hit_rate >= 49.0 && hit_rate <= 51.0);
    }
}
