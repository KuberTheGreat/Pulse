pub fn format_memory_kb(kb: u64) -> String{
    let mb = kb as f64 / (1024.0 * 1024.0);
    if mb < 1024.0{
        format!("{:.1} MB", mb)
    }else{
        format!("{:.2} GB",(mb / 1024.0))
    }
}

pub fn format_cpu(cpu: f32) -> String{
    format!("{:.1}%", cpu)
}