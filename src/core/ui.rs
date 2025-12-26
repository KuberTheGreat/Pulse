pub fn section(title: &str){
    println!();
    println!("{}", title);
    println!("{}", "-".repeat(title.len()));
}

pub fn kv(key: &str, value: &str){
    println!("{:<8} {}", format!("{}:", key), value);
}

pub fn bar(percent: f64, width: usize) -> String{
    let filled = ((percent / 100.0) * width as f64).round() as usize;
    let filled = filled.min(width);

    let bar = "█".repeat(filled) + &"░".repeat(width - filled);
    format!("{} {:>3.0}%", bar, percent)
}

pub fn sparkline(values: &[f64]) -> String{
    if values.is_empty(){
        return String::new();
    }

    let ticks = ['▁','▂','▃','▄','▅','▆','▇','█'];
    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    if (max - min).abs() < std::f64::EPSILON{
        return ticks[0].to_string().repeat(values.len());
    }

    values
        .iter()
        .map(|v| {
            let idx = (((v-min) / (max-min)) * (ticks.len() as f64 - 1.0))
                .round() as usize;
            ticks[idx]
        })
        .collect()
}