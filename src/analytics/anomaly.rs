//! Statistical anomaly detection using z-scores, EMA, and rolling variance.

use crate::storage::queries::ProcessMetricRow;

// ── Severity ─────────────────────────────────────────────────────────

/// Statistical severity level based on z-score magnitude.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Low => "LOW",
            Severity::Medium => "MEDIUM",
            Severity::High => "HIGH",
            Severity::Critical => "CRITICAL",
        }
    }

    pub fn from_zscore(z: f64) -> Self {
        if z > 4.0 {
            Severity::Critical
        } else if z > 3.0 {
            Severity::High
        } else if z > 2.0 {
            Severity::Medium
        } else {
            Severity::Low
        }
    }
}

// ── Anomaly result ───────────────────────────────────────────────────

#[derive(Debug)]
pub struct AnomalyResult {
    pub memory_anomaly: bool,
    pub cpu_anomaly: bool,
    pub memory_score: f64,
    pub cpu_score: f64,
    pub severity: Severity,
}

// ── Statistical helpers (shared across the analytics engine) ─────────

/// Arithmetic mean.
pub fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

/// Population standard deviation.
pub fn std_dev(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let m = mean(values);
    let variance = values.iter().map(|v| (v - m).powi(2)).sum::<f64>()
        / values.len() as f64;
    variance.sqrt()
}

/// Exponential Moving Average with smoothing factor `alpha` ∈ (0, 1].
pub fn ema(values: &[f64], alpha: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut result = values[0];
    for v in &values[1..] {
        result = alpha * v + (1.0 - alpha) * result;
    }
    result
}

/// Variance of the last `window` values (rolling).
pub fn rolling_variance(values: &[f64], window: usize) -> f64 {
    if values.len() < window || window == 0 {
        return 0.0;
    }
    let recent = &values[values.len() - window..];
    let m = mean(recent);
    recent.iter().map(|v| (v - m).powi(2)).sum::<f64>() / recent.len() as f64
}

// ── Detection ────────────────────────────────────────────────────────

/// Detect anomalies in `history` relative to `current_memory` and
/// `current_cpu`.  The caller supplies the z-score `threshold` (usually
/// 2.0) so it can be driven by config.
pub fn detect_anomaly(
    history: &[ProcessMetricRow],
    current_memory: u64,
    current_cpu: f32,
    threshold: f64,
) -> Option<AnomalyResult> {
    if history.len() < 5 {
        return None;
    }

    let mem_values: Vec<f64> = history.iter().map(|h| h.memory as f64).collect();
    let cpu_values: Vec<f64> = history.iter().map(|h| h.cpu).collect();

    let mem_mean = mean(&mem_values);
    let mem_std = std_dev(&mem_values);

    let cpu_mean = mean(&cpu_values);
    let cpu_std = std_dev(&cpu_values);

    let mem_score = if mem_std > 0.0 {
        ((current_memory as f64) - mem_mean).abs() / mem_std
    } else {
        0.0
    };

    let cpu_score = if cpu_std > 0.0 {
        ((current_cpu as f64) - cpu_mean).abs() / cpu_std
    } else {
        0.0
    };

    let memory_anomaly = mem_score > threshold;
    let cpu_anomaly = cpu_score > threshold;
    let max_score = mem_score.max(cpu_score);

    Some(AnomalyResult {
        memory_anomaly,
        cpu_anomaly,
        memory_score: mem_score,
        cpu_score,
        severity: Severity::from_zscore(max_score),
    })
}
