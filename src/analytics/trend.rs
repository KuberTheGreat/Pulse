//! Slope-based trend detection for memory and CPU.

use crate::storage::queries::ProcessMetricRow;

/// Direction of a resource usage trend.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrendKind {
    Increasing,
    Decreasing,
    Stable,
}

/// Result of trend analysis over a window of samples.
#[derive(Debug)]
pub struct TrendResult {
    pub memory_trend: TrendKind,
    pub cpu_trend: TrendKind,
    pub memory_slope: f64,
    pub cpu_slope: f64,
    /// How many samples were actually used.
    pub sample_count: usize,
}

/// Compute the ordinary-least-squares slope of `values` treated as
/// equally-spaced time-series points.
pub fn slope(values: &[f64]) -> f64 {
    let n = values.len() as f64;
    if n < 2.0 {
        return 0.0;
    }

    let x_mean = (n - 1.0) / 2.0;
    let y_mean: f64 = values.iter().sum::<f64>() / n;

    let mut num = 0.0;
    let mut den = 0.0;

    for (i, y) in values.iter().enumerate() {
        let x = i as f64;
        num += (x - x_mean) * (y - y_mean);
        den += (x - x_mean).powi(2);
    }

    if den == 0.0 {
        0.0
    } else {
        num / den
    }
}

/// Detect trends in the most recent `window` samples of `history`.
/// Requires at least 5 data points.
pub fn detect_trend(
    history: &[ProcessMetricRow],
    window: usize,
) -> Option<TrendResult> {
    if history.len() < 5 {
        return None;
    }

    let recent = &history[history.len().saturating_sub(window)..];
    let sample_count = recent.len();

    let mem_values: Vec<f64> = recent.iter().map(|h| h.memory as f64).collect();
    let cpu_values: Vec<f64> = recent.iter().map(|h| h.cpu).collect();

    let mem_slope = slope(&mem_values);
    let cpu_slope = slope(&cpu_values);

    let classify = |s: f64| {
        if s > 0.5 {
            TrendKind::Increasing
        } else if s < -0.5 {
            TrendKind::Decreasing
        } else {
            TrendKind::Stable
        }
    };

    Some(TrendResult {
        memory_trend: classify(mem_slope),
        cpu_trend: classify(cpu_slope),
        memory_slope: mem_slope,
        cpu_slope,
        sample_count,
    })
}
