//! Anomaly prediction engine — estimates time-to-anomaly with
//! confidence scoring and human-readable reasons.

use super::trend::TrendKind;
use super::anomaly;
use crate::storage::queries::ProcessMetricRow;

/// A prediction about when a resource anomaly is likely to occur.
#[derive(Debug)]
pub struct Prediction {
    /// Estimated minutes until an anomaly threshold is crossed.
    pub minutes_to_anomaly: Option<f64>,
    /// Confidence in the prediction (0.0 – 1.0).
    pub confidence: f64,
    /// The underlying trend direction.
    pub trend: TrendKind,
    /// Human-readable explanation of why this prediction was made.
    pub reason: String,
}

/// Given a current value, an anomaly threshold, and a positive slope,
/// estimate how many minutes until the threshold is reached.
pub fn predict_anomaly(
    current_value: f64,
    anomaly_threshold: f64,
    slope_per_minute: f64,
) -> Option<f64> {
    if slope_per_minute <= 0.0 {
        return None;
    }

    let remaining = anomaly_threshold - current_value;
    if remaining <= 0.0 {
        return Some(0.0);
    }

    Some(remaining / slope_per_minute)
}

/// Build a full prediction from process history and trend data.
pub fn build_prediction(
    history: &[ProcessMetricRow],
    current_memory: u64,
    memory_slope: f64,
    memory_trend: TrendKind,
) -> Option<Prediction> {
    if history.len() < 5 {
        return None;
    }

    let mem_values: Vec<f64> = history.iter().map(|h| h.memory as f64).collect();
    let m = anomaly::mean(&mem_values);
    let sd = anomaly::std_dev(&mem_values);

    if sd == 0.0 {
        return None;
    }

    let anomaly_threshold = m + 2.0 * sd;
    let minutes = predict_anomaly(current_memory as f64, anomaly_threshold, memory_slope);

    let confidence = (memory_slope.abs() / sd).min(1.0);

    let reason = match memory_trend {
        TrendKind::Increasing => format!(
            "Consistent positive memory growth over {} samples",
            history.len(),
        ),
        TrendKind::Stable => "Memory usage is stable — no anomaly expected".into(),
        TrendKind::Decreasing => "Memory usage is decreasing".into(),
    };

    minutes.map(|eta| Prediction {
        minutes_to_anomaly: Some(eta),
        confidence,
        trend: memory_trend,
        reason,
    })
}
