//! 60fps Benchmark Suite for Loom Rendering Pipeline
//! 
//! Measures frame timing, identifies bottlenecks, validates 60fps compliance.

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Target frame rate
pub const TARGET_FPS: f64 = 60.0;
pub const TARGET_FRAME_TIME: Duration = Duration::from_nanos(16_666_667); // ~16.67ms

/// Benchmark scenario types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BenchmarkScenario {
    /// Simple layout (few elements)
    SimpleLayout,
    /// Complex layout (many elements, nesting)
    ComplexLayout,
    /// AI mode transition
    AiModeTransition,
    /// Traditional mode with DevTools
    TraditionalMode,
    /// Scroll with virtualization
    VirtualScroll,
    /// Image loading stress test
    ImageLoading,
    /// Combined stress test
    FullStress,
}

impl BenchmarkScenario {
    pub fn name(&self) -> &'static str {
        match self {
            Self::SimpleLayout => "simple_layout",
            Self::ComplexLayout => "complex_layout",
            Self::AiModeTransition => "ai_mode_transition",
            Self::TraditionalMode => "traditional_mode",
            Self::VirtualScroll => "virtual_scroll",
            Self::ImageLoading => "image_loading",
            Self::FullStress => "full_stress",
        }
    }
}

/// Frame timing measurement
#[derive(Debug, Clone)]
pub struct FrameMeasurement {
    pub frame_number: u64,
    pub start: Instant,
    pub end: Instant,
    pub layout_time: Duration,
    pub paint_time: Duration,
    pub composite_time: Duration,
}

impl FrameMeasurement {
    pub fn total_time(&self) -> Duration {
        self.end.duration_since(self.start)
    }
    
    pub fn total_ms(&self) -> f64 {
        self.total_time().as_secs_f64() * 1000.0
    }
    
    pub fn meets_target(&self) -> bool {
        self.total_time() <= TARGET_FRAME_TIME
    }
}

/// Benchmark results for a scenario
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub scenario: BenchmarkScenario,
    pub frames: Vec<FrameMeasurement>,
    pub duration: Duration,
}

impl BenchmarkResult {
    /// Average frame time in milliseconds
    pub fn avg_frame_time_ms(&self) -> f64 {
        if self.frames.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.frames.iter().map(|f| f.total_ms()).sum();
        sum / self.frames.len() as f64
    }
    
    /// Minimum frame time
    pub fn min_frame_time_ms(&self) -> f64 {
        self.frames.iter().map(|f| f.total_ms()).fold(f64::MAX, f64::min)
    }
    
    /// Maximum frame time
    pub fn max_frame_time_ms(&self) -> f64 {
        self.frames.iter().map(|f| f.total_ms()).fold(0.0, f64::max)
    }
    
    /// 95th percentile frame time
    pub fn p95_frame_time_ms(&self) -> f64 {
        if self.frames.is_empty() {
            return 0.0;
        }
        let mut times: Vec<f64> = self.frames.iter().map(|f| f.total_ms()).collect();
        times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let idx = (times.len() as f64 * 0.95) as usize;
        times[idx.min(times.len() - 1)]
    }
    
    /// 99th percentile frame time
    pub fn p99_frame_time_ms(&self) -> f64 {
        if self.frames.is_empty() {
            return 0.0;
        }
        let mut times: Vec<f64> = self.frames.iter().map(|f| f.total_ms()).collect();
        times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let idx = (times.len() as f64 * 0.99) as usize;
        times[idx.min(times.len() - 1)]
    }
    
    /// Percentage of frames meeting 60fps target
    pub fn fps_compliance_pct(&self) -> f64 {
        if self.frames.is_empty() {
            return 100.0;
        }
        let compliant = self.frames.iter().filter(|f| f.meets_target()).count();
        (compliant as f64 / self.frames.len() as f64) * 100.0
    }
    
    /// Achieved FPS (average)
    pub fn achieved_fps(&self) -> f64 {
        let avg_ms = self.avg_frame_time_ms();
        if avg_ms > 0.0 {
            1000.0 / avg_ms
        } else {
            0.0
        }
    }
    
    /// Frames that missed the target (dropped frames)
    pub fn dropped_frames(&self) -> usize {
        self.frames.iter().filter(|f| !f.meets_target()).count()
    }
    
    /// Generate report string
    pub fn report(&self) -> String {
        format!(
            "Benchmark: {}\n\
             Duration: {:.1}s\n\
             Frames: {}\n\
             Avg Frame Time: {:.2}ms\n\
             Min/Max: {:.2}ms / {:.2}ms\n\
             P95/P99: {:.2}ms / {:.2}ms\n\
             FPS Compliance: {:.1}%\n\
             Achieved FPS: {:.1}\n\
             Dropped Frames: {}",
            self.scenario.name(),
            self.duration.as_secs_f64(),
            self.frames.len(),
            self.avg_frame_time_ms(),
            self.min_frame_time_ms(),
            self.max_frame_time_ms(),
            self.p95_frame_time_ms(),
            self.p99_frame_time_ms(),
            self.fps_compliance_pct(),
            self.achieved_fps(),
            self.dropped_frames()
        )
    }
}

/// Performance bottleneck identification
#[derive(Debug, Clone)]
pub struct Bottleneck {
    pub stage: PipelineStage,
    pub avg_time_ms: f64,
    pub pct_of_frame: f64,
    pub severity: Severity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PipelineStage {
    Layout,
    Paint,
    Composite,
    JavaScript,
    Network,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// Full benchmark suite
pub struct BenchmarkSuite {
    results: HashMap<BenchmarkScenario, BenchmarkResult>,
    current_scenario: Option<BenchmarkScenario>,
    current_frames: Vec<FrameMeasurement>,
    start_time: Option<Instant>,
}

impl BenchmarkSuite {
    pub fn new() -> Self {
        Self {
            results: HashMap::new(),
            current_scenario: None,
            current_frames: Vec::new(),
            start_time: None,
        }
    }
    
    /// Start a benchmark scenario
    pub fn start_scenario(&mut self, scenario: BenchmarkScenario) {
        self.current_scenario = Some(scenario);
        self.current_frames.clear();
        self.start_time = Some(Instant::now());
    }
    
    /// Record a frame measurement
    pub fn record_frame(&mut self, measurement: FrameMeasurement) {
        self.current_frames.push(measurement);
    }
    
    /// End current scenario and save results
    pub fn end_scenario(&mut self) -> Option<BenchmarkResult> {
        let scenario = self.current_scenario.take()?;
        let duration = self.start_time.map(|s| s.elapsed()).unwrap_or_default();
        
        let result = BenchmarkResult {
            scenario,
            frames: std::mem::take(&mut self.current_frames),
            duration,
        };
        
        self.results.insert(scenario, result.clone());
        Some(result)
    }
    
    /// Run all standard benchmarks
    pub fn run_all<F>(&mut self, mut runner: F) -> &HashMap<BenchmarkScenario, BenchmarkResult>
    where
        F: FnMut(BenchmarkScenario) -> Vec<FrameMeasurement>,
    {
        let scenarios = [
            BenchmarkScenario::SimpleLayout,
            BenchmarkScenario::ComplexLayout,
            BenchmarkScenario::AiModeTransition,
            BenchmarkScenario::TraditionalMode,
            BenchmarkScenario::VirtualScroll,
        ];
        
        for scenario in scenarios {
            self.start_scenario(scenario);
            let frames = runner(scenario);
            for frame in frames {
                self.record_frame(frame);
            }
            self.end_scenario();
        }
        
        &self.results
    }
    
    /// Get result for a specific scenario
    pub fn get_result(&self, scenario: BenchmarkScenario) -> Option<&BenchmarkResult> {
        self.results.get(&scenario)
    }
    
    /// Identify bottlenecks from measurements
    pub fn identify_bottlenecks(&self) -> Vec<Bottleneck> {
        let mut bottlenecks = Vec::new();
        
        // Aggregate timing by stage across all results
        let mut stage_times: HashMap<PipelineStage, Vec<f64>> = HashMap::new();
        
        for result in self.results.values() {
            for frame in &result.frames {
                stage_times.entry(PipelineStage::Layout)
                    .or_default()
                    .push(frame.layout_time.as_secs_f64() * 1000.0);
                stage_times.entry(PipelineStage::Paint)
                    .or_default()
                    .push(frame.paint_time.as_secs_f64() * 1000.0);
                stage_times.entry(PipelineStage::Composite)
                    .or_default()
                    .push(frame.composite_time.as_secs_f64() * 1000.0);
            }
        }
        
        // Calculate averages and identify bottlenecks
        let target_ms = TARGET_FRAME_TIME.as_secs_f64() * 1000.0;
        
        for (stage, times) in stage_times {
            if times.is_empty() {
                continue;
            }
            
            let avg: f64 = times.iter().sum::<f64>() / times.len() as f64;
            let pct = (avg / target_ms) * 100.0;
            
            let severity = if pct > 50.0 {
                Severity::Critical
            } else if pct > 30.0 {
                Severity::High
            } else if pct > 15.0 {
                Severity::Medium
            } else {
                Severity::Low
            };
            
            if severity != Severity::Low {
                bottlenecks.push(Bottleneck {
                    stage,
                    avg_time_ms: avg,
                    pct_of_frame: pct,
                    severity,
                });
            }
        }
        
        // Sort by severity
        bottlenecks.sort_by(|a, b| {
            let severity_ord = |s: Severity| match s {
                Severity::Critical => 3,
                Severity::High => 2,
                Severity::Medium => 1,
                Severity::Low => 0,
            };
            severity_ord(b.severity).cmp(&severity_ord(a.severity))
        });
        
        bottlenecks
    }
    
    /// Generate full benchmark report
    pub fn full_report(&self) -> String {
        let mut report = String::from("=== Loom 60fps Benchmark Report ===\n\n");
        
        // Summary for each scenario
        for (scenario, result) in &self.results {
            report.push_str(&format!("{}:\n", scenario.name()));
            report.push_str(&format!("  Compliance: {:.1}%\n", result.fps_compliance_pct()));
            report.push_str(&format!("  Avg FPS: {:.1}\n", result.achieved_fps()));
            report.push_str(&format!("  P95: {:.2}ms\n", result.p95_frame_time_ms()));
            report.push_str(&format!("  Dropped: {}\n\n", result.dropped_frames()));
        }
        
        // Bottlenecks
        let bottlenecks = self.identify_bottlenecks();
        if !bottlenecks.is_empty() {
            report.push_str("Bottlenecks:\n");
            for b in bottlenecks {
                report.push_str(&format!(
                    "  {:?}: {:.2}ms ({:.1}%) - {:?}\n",
                    b.stage, b.avg_time_ms, b.pct_of_frame, b.severity
                ));
            }
        }
        
        report
    }
}

/// Simple FPS counter for runtime monitoring
pub struct FpsCounter {
    frame_times: VecDeque<Instant>,
    window_size: usize,
}

impl FpsCounter {
    pub fn new(window_size: usize) -> Self {
        Self {
            frame_times: VecDeque::with_capacity(window_size),
            window_size,
        }
    }
    
    /// Record a frame
    pub fn tick(&mut self) {
        let now = Instant::now();
        
        if self.frame_times.len() >= self.window_size {
            self.frame_times.pop_front();
        }
        self.frame_times.push_back(now);
    }
    
    /// Get current FPS
    pub fn fps(&self) -> f64 {
        if self.frame_times.len() < 2 {
            return 0.0;
        }
        
        let first = self.frame_times.front().unwrap();
        let last = self.frame_times.back().unwrap();
        let duration = last.duration_since(*first).as_secs_f64();
        
        if duration > 0.0 {
            (self.frame_times.len() as f64 - 1.0) / duration
        } else {
            0.0
        }
    }
    
    /// Check if maintaining 60fps
    pub fn is_60fps(&self) -> bool {
        self.fps() >= 58.0 // Allow small margin
    }
}

use std::collections::VecDeque;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_benchmark_result_calculations() {
        let frames: Vec<FrameMeasurement> = (0..60)
            .map(|i| FrameMeasurement {
                frame_number: i,
                start: Instant::now(),
                end: Instant::now() + Duration::from_millis(16),
                layout_time: Duration::from_millis(5),
                paint_time: Duration::from_millis(8),
                composite_time: Duration::from_millis(3),
            })
            .collect();
        
        let result = BenchmarkResult {
            scenario: BenchmarkScenario::SimpleLayout,
            frames,
            duration: Duration::from_secs(1),
        };
        
        assert!(result.fps_compliance_pct() > 99.0);
        assert!(result.achieved_fps() > 59.0);
    }
    
    #[test]
    fn test_fps_counter() {
        let mut counter = FpsCounter::new(60);
        
        for _ in 0..10 {
            counter.tick();
            std::thread::sleep(Duration::from_millis(16));
        }
        
        let fps = counter.fps();
        assert!(fps > 0.0);
    }
}
