//! Loom Benchmark Crate
//!
//! Performance testing and benchmarking infrastructure for Loom browser.
//! Provides FPS counters, frame time histograms, and GPU/CPU split tracking.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod fps_counter;
pub mod baseline;

pub use fps_counter::{
    FpsCounter, 
    FrameStats, 
    SplitTime, 
    FrameMeasurement,
    FrameTimer,
};

use alloc::string::String;
use alloc::vec::Vec;

/// Benchmark result summary
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    /// Benchmark name
    pub name: String,
    /// Total frames rendered
    pub total_frames: usize,
    /// Duration of benchmark
    pub duration_secs: f64,
    /// Average FPS
    pub avg_fps: f64,
    /// Minimum FPS
    pub min_fps: f64,
    /// Maximum FPS
    pub max_fps: f64,
    /// Frame time statistics
    pub frame_stats: FrameStats,
    /// Target hit rate (% frames achieving target FPS)
    pub target_hit_rate: f64,
    /// GPU vs CPU split
    pub gpu_cpu_split: SplitTime,
}

impl BenchmarkResult {
    /// Format as human-readable report
    pub fn format_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str(&format!("=== {} Benchmark Results ===\n", self.name));
        report.push_str(&format!("Duration:      {:.2}s\n", self.duration_secs));
        report.push_str(&format!("Total frames:  {}\n", self.total_frames));
        report.push_str(&format!("Average FPS:   {:.1}\n", self.avg_fps));
        report.push_str(&format!("Min/Max FPS:   {:.1} / {:.1}\n", self.min_fps, self.max_fps));
        report.push_str(&format!("Target hit:    {:.1}%\n", self.target_hit_rate));
        
        report.push_str("\n--- Frame Times ---\n");
        report.push_str(&format!("  p50:  {:?}\n", self.frame_stats.p50));
        report.push_str(&format!("  p95:  {:?}\n", self.frame_stats.p95));
        report.push_str(&format!("  p99:  {:?}\n", self.frame_stats.p99));
        report.push_str(&format!("  mean: {:?}\n", self.frame_stats.mean));
        
        report.push_str("\n--- GPU/CPU Split ---\n");
        report.push_str(&format!("  CPU:  {:?}\n", self.gpu_cpu_split.cpu));
        report.push_str(&format!("  GPU:  {:?}\n", self.gpu_cpu_split.gpu));
        
        let cpu_pct = if !self.gpu_cpu_split.total.is_zero() {
            self.gpu_cpu_split.cpu.as_nanos() as f64 / 
            self.gpu_cpu_split.total.as_nanos() as f64 * 100.0
        } else {
            0.0
        };
        report.push_str(&format!("  CPU%: {:.1}%\n", cpu_pct));
        
        report
    }
}

/// Test page configuration for benchmarks
#[derive(Debug, Clone)]
pub struct TestPage {
    /// Page name
    pub name: String,
    /// HTML content
    pub html: String,
    /// CSS content
    pub css: String,
    /// Expected complexity (element count)
    pub element_count: usize,
}

impl TestPage {
    /// Simple static page
    pub fn simple_static() -> Self {
        Self {
            name: String::from("simple_static"),
            html: String::from(r#"
                <html><body>
                    <h1>Hello World</h1>
                    <p>This is a simple static page.</p>
                </body></html>
            "#),
            css: String::from(r#"
                body { font-family: sans-serif; margin: 20px; }
                h1 { color: #333; }
            "#),
            element_count: 10,
        }
    }

    /// Complex layout page
    pub fn complex_layout() -> Self {
        Self {
            name: String::from("complex_layout"),
            html: String::from(r#"
                <html><body>
                    <div class="container">
                        <header><nav><ul><li>Home</li><li>About</li></ul></nav></header>
                        <main>
                            <article><h1>Title</h1><p>Content...</p></article>
                            <aside><h2>Sidebar</h2><p>More content...</p></aside>
                        </main>
                        <footer><p>Footer</p></footer>
                    </div>
                </body></html>
            "#),
            css: String::from(r#"
                .container { display: flex; flex-direction: column; }
                main { display: flex; }
                article { flex: 2; }
                aside { flex: 1; }
            "#),
            element_count: 50,
        }
    }

    /// Stress test page with many elements
    pub fn stress_test() -> Self {
        // Generate 1000 list items
        let mut html = String::from("<html><body><ul>");
        for i in 0..1000 {
            html.push_str(&format!("<li>Item {}</li>", i));
        }
        html.push_str("</ul></body></html>");

        Self {
            name: String::from("stress_test"),
            html,
            css: String::from(r#"
                ul { list-style: none; padding: 0; }
                li { padding: 5px; border-bottom: 1px solid #ccc; }
                li:nth-child(odd) { background: #f5f5f5; }
            "#),
            element_count: 1005,
        }
    }
}

/// Standard test pages for benchmarking
pub fn standard_test_pages() -> Vec<TestPage> {
    vec![
        TestPage::simple_static(),
        TestPage::complex_layout(),
        TestPage::stress_test(),
    ]
}

/// Version of the benchmark crate
pub const VERSION: &str = "0.1.0-L29";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_result_format() {
        let result = BenchmarkResult {
            name: String::from("test"),
            total_frames: 60,
            duration_secs: 1.0,
            avg_fps: 60.0,
            min_fps: 58.0,
            max_fps: 62.0,
            frame_stats: FrameStats::default(),
            target_hit_rate: 95.0,
            gpu_cpu_split: SplitTime::default(),
        };

        let report = result.format_report();
        assert!(report.contains("test Benchmark Results"));
        assert!(report.contains("60.0"));
    }

    #[test]
    fn test_test_pages() {
        let pages = standard_test_pages();
        assert_eq!(pages.len(), 3);
        
        assert_eq!(pages[0].name, "simple_static");
        assert_eq!(pages[1].name, "complex_layout");
        assert_eq!(pages[2].name, "stress_test");
        
        assert!(pages[2].element_count > 1000);
    }
}
