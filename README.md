<div align="center">
  <h1>⚡ Pulse</h1>
  <p><strong>Intelligent System Behavior Analysis & Observability Platform</strong></p>
</div>

Pulse is a lightweight, high-performance Rust-based observability tool that analyzes system processes, learns their historical behavior, predicts anomalies, and explains what is happening in plain English.

Unlike traditional monitoring tools that only display raw metrics, Pulse focuses on **behavioral understanding** — utilizing statistical analytics, background data collection, and trend evaluation to tell you *why* a process is misbehaving before it causes a system crash.

---

## ✨ Key Features

- **Background Daemon (`pulse-daemon`)**: Collects system and process metrics continuously with `< 1%` CPU overhead.
- **SQLite Storage**: Highly performant, WAL-mode SQLite database stores historical metrics across days or months.
- **Statistical Anomaly Engine**: Uses Z-scores, Exponential Moving Averages (EMA), and rolling variance to accurately flag abnormal CPU or memory behavior.
- **Memory Leak Prediction**: Evaluates slopes of metric history to predict Time-To-Anomaly (ETA) with confidence scoring.
- **Desktop Alerts**: Sends cross-platform native notifications for threshold breaches or leak predictions, complete with an anti-spam cooldown manager.
- **Human-Readable Insights**: Converts mathematical analytics into plain English explanations.
- **Sparkline Visualizations**: Rich CLI terminal UI with progress bars, status badges, and inline sparkline graphs.

---

## 🛠️ Tech Stack

- **Language:** Rust (2024 edition)
- **CLI Parsing:** `clap`
- **System Metrics:** `sysinfo`
- **Database Engine:** `rusqlite` (bundled SQLite 3)
- **Configuration:** `toml` (`~/.pulse/config.toml`)
- **Logging:** `env_logger` & `log`
- **Signal Handling:** `ctrlc` for graceful shutdown

---

## 🚀 Installation & Setup

1. **Clone & Build:**
   ```bash
   git clone https://github.com/your-username/pulse.git
   cd pulse
   cargo build --release
   ```

2. **Run the Background Daemon:**
   The daemon runs quietly in the background, sampling processes and performing analytics.
   ```bash
   ./target/release/pulse daemon start
   ```

3. **Check the System Dashboard:**
   ```bash
   ./target/release/pulse status
   ```

*(Optionally, move the `pulse` and `pulse-daemon` binaries to a directory in your `$PATH` like `/usr/local/bin`)*

---

## 💻 CLI Commands

Pulse includes a comprehensive set of subcommands to inspect your system:

| Command | Description |
|---------|-------------|
| `pulse status` | View a global dashboard of CPU, Memory, Swap, and recent history trends. |
| `pulse process <name\|PID>` | Inspect a specific process. Shows sparkline history, anomaly detection, and ETA predictions. |
| `pulse summary <name>` | Evaluates the long-term anomaly frequency and risk level of a given process. |
| `pulse anomalies` | List the most recently detected anomalies stored in the database. |
| `pulse history` | View tabular historical data for the system (or pass `--process <name>`). |
| `pulse doctor` | Perform a health check on the daemon, database size, and configuration file. |
| `pulse config` | Print the currently active configuration. |
| `pulse daemon start` | Launch the background metrics collector. |
| `pulse daemon stop` | Gracefully shut down the background daemon. |
| `pulse daemon status` | Check if the daemon is currently running. |

### 📸 Command Output Previews

**Process Inspection with Prediction Engine:**
```text
  ⚡ Pulse — Intelligent System Behavior Analysis
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  Process: WindowServer (PID 182)  
───────────────────────────────────
  Memory:        █████████░░░░░░░░░░░  1.42 GB
  CPU:           ██████████████████░░  88.5%

  Trend Analysis  
──────────────────
  Memory:        📈 Increasing (possible leak)
  CPU:           ➖ Stable
  Samples:       120

  Prediction  
──────────────
  ETA:           42 minutes
  Confidence:    █████████████░░  86.0%
  Trend:         Increasing
  Reason:        Consistent positive memory growth over 120 samples
```

---

## ⚙️ Configuration

On its first run, Pulse generates a default configuration file at `~/.pulse/config.toml`. You can modify these parameters to tune Pulse to your machine.

```toml
# Collection interval in seconds (daemon)
interval = 5

# Days of history to retain before automatic cleanup
history_days = 30

# CPU usage alert threshold (%)
cpu_threshold = 90.0

# Memory usage alert threshold (%)
memory_threshold = 85.0

# Enable desktop notifications
enable_notifications = true

# Minimum seconds between repeated alerts (anti-spam)
notification_cooldown = 300

# Top-N processes to track per cycle
top_n = 15
```

---

## 🏗️ Architecture Layout

- `bin/pulse.rs` - The CLI interface for visualizing data and sending commands.
- `bin/daemon.rs` - The headless daemon responsible for metric collection and analytics.
- `storage/` - SQLite database wrapper, automated migrations, and prepared queries.
- `analytics/` - Pure statistical analysis (Z-score, Variance, EMA, Linear Regression, ETA calculations).
- `collector/` - Reusable `sysinfo` wrappers optimized for `< 1%` CPU footprint.
- `ui/` - ANSI-colored terminal widgets (bars, sparklines, badges).

---

## 📄 License
MIT License
