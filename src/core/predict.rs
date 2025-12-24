use crate::core::anomaly::AnomalyResult;
use crate::core::trend::TrendResult;

#[derive(Debug)]
pub struct Prediction{
    pub minutes_to_anomaly: Option<f64>,
    pub confidence: f64,
}

pub fn predict_anomaly(
    current_value: f64,
    anomaly_threshold: f64,
    slope_per_minute: f64,
) -> Option<f64> {
    if slope_per_minute <= 0.0{
        return None;
    }

    let remaining = anomaly_threshold - current_value;

    if remaining <= 0.0{
        return Some(0.0);
    }

    Some(remaining / slope_per_minute)

}

pub fn build_prediction(
    current: f64,
    mean: f64,
    std_dev: f64,
    slope: f64,
) -> Option<Prediction>{
    if std_dev == 0.0{
        return None;
    }

    let anomaly_threshold = mean + (2.0 * std_dev);
    let minutes = predict_anomaly(current, anomaly_threshold, slope);

    minutes.map(|m| Prediction { minutes_to_anomaly: Some(m), confidence: (slope.abs() / std_dev).min(1.0), })
}