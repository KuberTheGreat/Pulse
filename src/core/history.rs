use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessHistoryEntry{
    pub memory: u64,
    pub cpu_usage: f32,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct HistoryStore{
    pub processes: HashMap<String, Vec<ProcessHistoryEntry>>,
}

fn history_path() -> PathBuf{
    let mut path = dirs::home_dir().expect("Could not find home directory");
    path.push(".pulse");
    fs::create_dir_all(&path).ok();
    path.push("history.json");
    path
}

pub fn load_history() -> HistoryStore{
    let path = history_path();

    if !path.exists(){
        return HistoryStore::default();
    }

    let data = fs::read_to_string(path).unwrap_or_default();
    serde_json::from_str(&data).unwrap_or_default()
}

pub fn save_history(history: &HistoryStore){
    let path = history_path();
    let data = serde_json::to_string_pretty(history).unwrap();
    fs::write(path, data).unwrap();
}

pub fn now_ts() -> i64{
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}