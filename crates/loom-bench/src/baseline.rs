//! L29 Performance Baseline Report Generator
//!
//! Generates initial performance baseline reports with:
//! - Current FPS measurements on test pages
//! - Bottleneck identification
//! - Optimization targets

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::format;

use crate::{FpsCounter, TestPage, standard_test_pages, FrameStats};
use core::time::Duration;

/// Performance grade based on FPS
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerformanceGrade {
    /// 60+ FPS (Excellent)
    Excellent,
    /// 45-59 FPS (Good)
    Good,
    /// 30-44 FPS (Acceptable)
    Acceptable,
    /// 15-29 FPS (Poor)
    Poor,
    /// <15 FPS (Unusable)
    Unusable,
}

impl PerformanceGrade {
    /// Grade from FPS value
    pub fn from_fps(fps: f64) -> Self {
        match fps {
            f if f >= 60.0 => PerformanceGrade::Excellent,
            f if f >= 45.0 => PerformanceGrade::Good,
            f if f >= 30.0 => PerformanceGrade::Acceptable,
            f if f >= 15.0 => PerformanceGrade::Poor,
            _ => PerformanceGrade::Unusable,
        }
    }

    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            PerformanceGrade::Excellent => "Excellent (60+ FPS)",
            PerformanceGrade::Good => "Good (45-59 FPS)",
            PerformanceGrade::Acceptable => "Acceptable (30-44 FPS)",
            PerformanceGrade::Poor => "Poor (15-29 FPS)",
            PerformanceGrade::Unusable => "Unusable (<15 FPS)",
        }
    }

    /// Get emoji indicator
    pub fn emoji(&self) -> &'static str {
        match self {
            PerformanceGrade::Excellent => "✅",
            PerformanceGrade::Good => "🟢",
            PerformanceGrade::Acceptable => "🟡",
            PerformanceGrade::Poor => "🔴",
            PerformanceGrade::Unusable => "⛔",
        }
    }

    /// Is this grade acceptable for production?
    pub fn is_acceptable(&self) -> bool {
        matches!(self, PerformanceGrade::Excellent | PerformanceGrade::Good)
    }
}

/// Single page benchmark result
#[derive(Debug, Clone)]
pub struct PageBenchmark {
    /// Test page
    pub page: TestPage,
    /// Average FPS
    pub avg_fps: f64,
    /// Performance grade
    pub grade: PerformanceGrade,
    /// Frame time statistics
    pub frame_stats: FrameStats,
    /// Target hit rate (%)
    pub target_hit_rate: f64,
    /// GPU time percentage
    pub gpu_percentage: f64,
}

/// Identified bottleneck
#[derive(Debug, Clone)]
pub struct Bottleneck {
    /// Description of the bottleneck
    pub description: String,
    /// Affected page/area
    pub location: String,
    /// Severity (1-5)
    pub severity: u8,
    /// Estimated impact on FPS
    pub fps_impact: f64,
    /// Recommended fix
    pub recommendation: String,
}

/// Optimization target
#[derive(Debug, Clone)]
pub struct OptimizationTarget {
    /// Target name
    pub name: String,
    /// Current state
    pub current: String,
    /// Target state
    pub target: String,
    /// Priority (1-5)
    pub priority: u8,
    /// Estimated effort (hours)
    pub effort_hours: u32,
    /// Expected FPS improvement
    pub fps_improvement: f64,
}

/// Complete baseline report
#[derive(Debug, Clone)]
pub struct BaselineReport {
    /// Report generation timestamp
    pub timestamp: String,
    /// Loom version
    pub loom_version: String,
    /// Page benchmarks
    pub page_results: Vec<PageBenchmark>,
    /// Identified bottlenecks
    pub bottlenecks: Vec<Bottleneck>,
    /// Optimization targets
    pub targets: Vec<OptimizationTarget>,
    /// Overall grade
    pub overall_grade: PerformanceGrade,
    /// Summary statistics
    pub summary: SummaryStats,
}

/// Summary statistics
#[derive(Debug, Clone)]
pub struct SummaryStats {
    /// Average FPS across all pages
    pub overall_avg_fps: f64,
    /// Best performing page
    pub best_page: String,
    pub best_fps: f64,
    /// Worst performing page
    pub worst_page: String,
    pub worst_fps: f64,
    /// Number of pages meeting 60 FPS target
    pub pages_at_60fps: usize,
    /// Number of pages meeting 30 FPS minimum
    pub pages_at_30fps: usize,
}

impl BaselineReport {
    /// Generate full markdown report
    pub fn generate_markdown(&self) -> String {
        let mut md = String::new();

        // Header
        md.push_str("# L29 Performance Baseline Report\n\n");
        md.push_str(&format!("**Generated:** {}\n\n", self.timestamp));
        md.push_str(&format!("**Loom Version:** {}\n\n", self.loom_version));

        // Executive Summary
        md.push_str("## Executive Summary\n\n");
        md.push_str(&format!("{} **Overall Grade:** {}\n\n", 
            self.overall_grade.emoji(), 
            self.overall_grade.name()));
        
        md.push_str("| Metric | Value |\n");
        md.push_str("|:---|:---|\n");
        md.push_str(&format!("| Average FPS | {:.1} |\n", self.summary.overall_avg_fps));
        md.push_str(&format!("| Pages at 60 FPS | {}/{} |\n", 
            self.summary.pages_at_60fps, self.page_results.len()));
        md.push_str(&format!("| Pages at 30+ FPS | {}/{} |\n",
            self.summary.pages_at_30fps, self.page_results.len()));
        md.push_str(&format!("| Best Page | {} ({:.1} FPS) |\n", 
            self.summary.best_page, self.summary.best_fps));
        md.push_str(&format!("| Worst Page | {} ({:.1} FPS) |\n",
            self.summary.worst_page, self.summary.worst_fps));
        md.push('\n');

        // Page Benchmarks
        md.push_str("## Page Benchmarks\n\n");
        for result in &self.page_results {
            md.push_str(&format!("### {}\n\n", result.page.name));
            md.push_str(&format!("{} **Grade:** {}\n\n", 
                result.grade.emoji(), result.grade.name()));
            
            md.push_str("| Metric | Value |\n");
            md.push_str("|:---|:---|\n");
            md.push_str(&format!("| Average FPS | {:.1} |\n", result.avg_fps));
            md.push_str(&format!("| Target Hit Rate | {:.1}% |\n", result.target_hit_rate));
            md.push_str(&format!("| p50 Frame Time | {:?} |\n", result.frame_stats.p50));
            md.push_str(&format!("| p95 Frame Time | {:?} |\n", result.frame_stats.p95));
            md.push_str(&format!("| p99 Frame Time | {:?} |\n", result.frame_stats.p99));
            md.push_str(&format!("| GPU Time | {:.1}% |\n", result.gpu_percentage));
            md.push('\n');
        }

        // Bottlenecks
        md.push_str("## Identified Bottlenecks\n\n");
        if self.bottlenecks.is_empty() {
            md.push_str("No major bottlenecks identified.\n\n");
        } else {
            for (i, bottleneck) in self.bottlenecks.iter().enumerate() {
                md.push_str(&format!("### {}. {}\n\n", i + 1, bottleneck.description));
                md.push_str(&format!("- **Location:** {}\n", bottleneck.location));
                md.push_str(&format!("- **Severity:** {}/5\n", bottleneck.severity));
                md.push_str(&format!("- **FPS Impact:** {:.1}\n", bottleneck.fps_impact));
                md.push_str(&format!("- **Recommendation:** {}\n\n", bottleneck.recommendation));
            }
        }

        // Optimization Targets
        md.push_str("## Optimization Targets\n\n");
        md.push_str("| Priority | Target | Current → Target | Effort | FPS Gain |\n");
        md.push_str("|:---|:---|:---|:---|:---|\n");
        
        let mut targets = self.targets.clone();
        targets.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        for target in targets {
            let priority_str = match target.priority {
                5 => "🔴 Critical",
                4 => "🟠 High",
                3 => "🟡 Medium",
                2 => "🔵 Low",
                _ => "⚪ Optional",
            };
            md.push_str(&format!("| {} | {} | {} → {} | {}h | +{:.1} |\n",
                priority_str,
                target.name,
                target.current,
                target.target,
                target.effort_hours,
                target.fps_improvement));
        }
        md.push('\n');

        // Next Steps
        md.push_str("## Recommended Next Steps\n\n");
        md.push_str("1. Address critical bottlenecks first (severity 4-5)\n");
        md.push_str("2. Implement React Fiber work loop (L29-FIBER)\n");
        md.push_str("3. Optimize slowest pipeline stage identified above\n");
        md.push_str("4. Re-run baseline after each optimization\n\n");

        md.push_str("---\n\n");
        md.push_str("*Report generated by loom-bench v0.1.0-L29*\n");

        md
    }

    /// Generate JSON report (for CI/CD)
    pub fn generate_json(&self) -> String {
        // Simple JSON serialization
        let mut json = String::from("{\n");
        json.push_str(&format!("  \"timestamp\": \"{}\",\n", self.timestamp));
        json.push_str(&format!("  \"loom_version\": \"{}\",\n", self.loom_version));
        json.push_str(&format!("  \"overall_grade\": \"{}\",\n", self.overall_grade.name()));
        json.push_str(&format!("  \"overall_avg_fps\": {:.2},\n", self.summary.overall_avg_fps));
        json.push_str(&format!("  \"pages_at_60fps\": {},\n", self.summary.pages_at_60fps));
        json.push_str(&format!("  \"pages_at_30fps\": {},\n", self.summary.pages_at_30fps));
        json.push_str(&format!("  \"bottleneck_count\": {},\n", self.bottlenecks.len()));
        json.push_str(&format!("  \"optimization_targets\": {}\n", self.targets.len()));
        json.push('}');
        json
    }
}

/// Baseline report generator
pub struct BaselineGenerator {
    pages: Vec<TestPage>,
}

impl BaselineGenerator {
    /// Create with standard test pages
    pub fn new() -> Self {
        Self {
            pages: standard_test_pages(),
        }
    }

    /// Create with custom pages
    pub fn with_pages(pages: Vec<TestPage>) -> Self {
        Self { pages }
    }

    /// Generate baseline report (placeholder - would integrate with actual renderer)
    pub fn generate(&self) -> BaselineReport {
        // In real implementation, this would:
        // 1. Render each test page
        // 2. Run FPS counter for N frames
        // 3. Collect actual measurements
        // 4. Analyze bottlenecks

        // Placeholder results for structure demonstration
        let page_results: Vec<PageBenchmark> = self.pages.iter().map(|page| {
            let element_factor = page.element_count as f64 / 100.0;
            let estimated_fps = 60.0 / element_factor.max(1.0);
            
            PageBenchmark {
                page: page.clone(),
                avg_fps: estimated_fps,
                grade: PerformanceGrade::from_fps(estimated_fps),
                frame_stats: FrameStats::default(),
                target_hit_rate: if estimated_fps >= 60.0 { 95.0 } else { 60.0 },
                gpu_percentage: 40.0,
            }
        }).collect();

        // Calculate summary
        let avg_fps: f64 = page_results.iter().map(|p| p.avg_fps).sum::<f64>() 
            / page_results.len() as f64;
        
        let best = page_results.iter().max_by(|a, b| 
            a.avg_fps.partial_cmp(&b.avg_fps).unwrap()).unwrap();
        let worst = page_results.iter().min_by(|a, b| 
            a.avg_fps.partial_cmp(&b.avg_fps).unwrap()).unwrap();
        
        let at_60 = page_results.iter().filter(|p| p.avg_fps >= 60.0).count();
        let at_30 = page_results.iter().filter(|p| p.avg_fps >= 30.0).count();

        // Identify bottlenecks (placeholder logic)
        let mut bottlenecks = Vec::new();
        
        if worst.avg_fps < 30.0 {
            bottlenecks.push(Bottleneck {
                description: String::from("High element count causing layout thrashing"),
                location: worst.page.name.clone(),
                severity: 4,
                fps_impact: 60.0 - worst.avg_fps,
                recommendation: String::from("Implement virtual scrolling, optimize layout algorithm"),
            });
        }

        // Create optimization targets
        let targets = vec![
            OptimizationTarget {
                name: String::from("React Fiber Work Loop"),
                current: String::from("Synchronous rendering"),
                target: String::from("Time-sliced, pausable rendering"),
                priority: 5,
                effort_hours: 40,
                fps_improvement: 15.0,
            },
            OptimizationTarget {
                name: String::from("Layout Cache"),
                current: String::from("Full layout every frame"),
                target: String::from("Incremental layout with dirty tracking"),
                priority: 4,
                effort_hours: 20,
                fps_improvement: 10.0,
            },
            OptimizationTarget {
                name: String::from("GPU Batch Optimization"),
                current: String::from("Individual draw calls"),
                target: String::from("Batched rendering"),
                priority: 3,
                effort_hours: 15,
                fps_improvement: 8.0,
            },
        ];

        BaselineReport {
            timestamp: String::from("2026-03-06"),
            loom_version: String::from("0.1.0-L29"),
            page_results,
            bottlenecks,
            targets,
            overall_grade: PerformanceGrade::from_fps(avg_fps),
            summary: SummaryStats {
                overall_avg_fps: avg_fps,
                best_page: best.page.name.clone(),
                best_fps: best.avg_fps,
                worst_page: worst.page.name.clone(),
                worst_fps: worst.avg_fps,
                pages_at_60fps: at_60,
                pages_at_30fps: at_30,
            },
        }
    }
}

impl Default for BaselineGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_grade() {
        assert_eq!(PerformanceGrade::from_fps(75.0), PerformanceGrade::Excellent);
        assert_eq!(PerformanceGrade::from_fps(55.0), PerformanceGrade::Good);
        assert_eq!(PerformanceGrade::from_fps(35.0), PerformanceGrade::Acceptable);
        assert_eq!(PerformanceGrade::from_fps(20.0), PerformanceGrade::Poor);
        assert_eq!(PerformanceGrade::from_fps(10.0), PerformanceGrade::Unusable);
    }

    #[test]
    fn test_baseline_generation() {
        let generator = BaselineGenerator::new();
        let report = generator.generate();

        assert!(!report.page_results.is_empty());
        assert!(!report.targets.is_empty());
        
        let md = report.generate_markdown();
        assert!(md.contains("Performance Baseline Report"));
        assert!(md.contains("Page Benchmarks"));
        assert!(md.contains("Optimization Targets"));
    }

    #[test]
    fn test_json_generation() {
        let generator = BaselineGenerator::new();
        let report = generator.generate();
        let json = report.generate_json();

        assert!(json.contains("timestamp"));
        assert!(json.contains("overall_grade"));
        assert!(json.contains("overall_avg_fps"));
    }
}
