use crate::core::history::ProcessHistoryEntry;

#[derive(Debug)]
pub struct AnomalyResult{
    pub memory_anomaly: bool,
    pub cpu_anomaly: bool,
    pub memory_score: f64,
    pub cpu_score: f64,
}

fn mean(values: &[f64]) -> f64{
    values.iter().sum::<f64>() / values.len() as f64
}

fn std_dev(values: &[f64], mean: f64) -> f64{
    let variance = values
        .iter()
        .map(|v| (v - mean).powi(2))
        .sum::<f64>() / values.len() as f64;

    variance.sqrt()
}

pub fn detect_anomaly(history:&[ProcessHistoryEntry], current_memory: u64, current_cpu: f32,) -> Option<AnomalyResult>{
    if history.len() < 5{
        return None;
    }

    let mem_values: Vec<f64> = history.iter().map(|h| h.memory as f64).collect();
    let cpu_values: Vec<f64> = history.iter().map(|h| h.cpu_usage as f64).collect();

    let mem_mean = mean(&mem_values);
    let mem_std = std_dev(&mem_values, mem_mean);

    let cpu_mean = mean(&cpu_values);
    let cpu_std = std_dev(&cpu_values, cpu_mean);

    let mem_score = if mem_std > 0.0 {
        ((current_memory as f64) - mem_mean).abs() / mem_std
    } else{
        0.0
    };

    let cpu_score = if cpu_std > 0.0{
        ((current_cpu as f64) - cpu_mean).abs() / cpu_std
    }else{
        0.0
    };

    Some(AnomalyResult { memory_anomaly: mem_score>2.0, cpu_anomaly: cpu_score>2.0, memory_score: mem_score, cpu_score, })
}