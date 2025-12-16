use sysinfo::{System};

#[derive(Debug)]
pub struct SystemSnapshot{
    pub total_memory: u64,
    pub used_memory: u64,
    pub cpu_usage: f32,
}

#[derive(Debug)]
pub struct ProcessSnapshot{
    pub pid: i32,
    pub name: String,
    pub memory: u64,
    pub cpu_usage: f32,
}

pub fn system_snapshot() -> SystemSnapshot{
    let mut system = System::new_all();
    system.refresh_all();

    SystemSnapshot{
        total_memory: system.total_memory(),
        used_memory: system.used_memory(),
        cpu_usage: system.global_cpu_info().cpu_usage(),
    }
}

pub fn inspect_process(name: &str) -> Vec<ProcessSnapshot> {
    let mut system = System::new_all();
    system.refresh_all();

    let mut results = Vec::new();

    for (pid, process) in system.processes(){
        if process.name().eq_ignore_ascii_case(name){
            results.push(ProcessSnapshot{
                pid: pid.as_u32() as i32,
                name: process.name().to_string(),
                memory: process.memory(),
                cpu_usage: process.cpu_usage(),
            });
        }
    }

    results
}