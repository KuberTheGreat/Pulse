# Pulse ⚡  
**Intelligent System Behavior Analysis CLI**

Pulse is a Rust-based CLI tool that analyzes system processes, learns their historical behavior, detects anomalies, identifies long-term trends, and explains what is happening in plain English.

Unlike traditional monitoring tools that only display raw metrics, Pulse focuses on **behavioral understanding** — helping users answer *whether something is normal, why it is abnormal, and how it is evolving over time*.

---

## ✨ Key Features

### 📊 Live Metrics
- System-wide CPU and memory usage
- Per-process inspection
- macOS-friendly, user-space only

### 🧠 Persistent Process History
- Learns how each process typically behaves
- Stores CPU and memory usage over time
- Data persists across runs

### 🚨 Statistical Anomaly Detection
- Uses **z-score–based analysis**
- Compares current behavior against historical baselines
- Flags abnormal CPU or memory usage

### 📈 Trend Detection
- Detects gradual behavior changes (e.g., memory leaks)
- Differentiates sudden spikes from long-term growth
- Uses slope-based trend estimation

### 💬 Human-Readable Explanations
- Converts statistical results into plain English
- Explains *why* a process is considered abnormal
- Fully explainable, no black-box ML

---

## 🛠️ Tech Stack

- **Language:** Rust (2021 edition)
- **CLI:** clap
- **System Metrics:** sysinfo
- **Serialization:** serde / serde_json
- **Storage:** Local JSON (upgradeable to SQLite)

