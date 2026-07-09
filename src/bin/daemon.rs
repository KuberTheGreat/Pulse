//! `pulse-daemon` — background metric collection daemon.

use pulse::config::PulseConfig;
use pulse::storage::Database;
use pulse::daemon::DaemonRunner;
use pulse::logging;

fn main() {
    let config = PulseConfig::load();
    if let Err(e) = config.validate() {
        eprintln!("Configuration error: {}", e);
        std::process::exit(1);
    }

    logging::init_logging(&config.log_level);

    // Ensure config template exists
    PulseConfig::generate_default_config();

    // Migrate old JSON history if present
    pulse::storage::migrate_from_json(&config);

    let db = match Database::open(&config.db_path()) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Failed to open database: {}", e);
            std::process::exit(1);
        }
    };

    let mut runner = DaemonRunner::new(config, db);
    runner.run();
}
