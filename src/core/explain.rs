use crate::core::anomaly::AnomalyResult;

pub fn explain_anomaly(process: &str, result: &AnomalyResult,) -> Vec<String>{
    let mut explanations = Vec::new();

    if result.memory_anomaly{
        if result.memory_score > 3.0 {
            explanations.push(format!("Memory usage for '{}' is extremely high compared to its normal behavior.", process));
        }else{
            explanations.push(format!("Memory usage for '{}' is significantly higher than usual", process));
        }
    }

    if result.cpu_anomaly {
        if result.cpu_score > 3.0 {
            explanations.push(format!(
                "CPU usage for '{}' spiked far beyond its typical usage pattern.",
                process
            ));
        } else {
            explanations.push(format!(
                "CPU usage for '{}' is noticeably higher than normal.",
                process
            ));
        }
    }

    if explanations.is_empty() {
        explanations.push(format!(
            "'{}' is behaving within its normal historical range.",
            process
        ));
    }

    explanations
}