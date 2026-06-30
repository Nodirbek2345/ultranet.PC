use rusqlite::{Connection, Result, params};
use std::sync::Mutex;
use lazy_static::lazy_static;
use std::path::PathBuf;

lazy_static! {
    static ref DB_CONN: Mutex<Option<Connection>> = Mutex::new(None);
}

pub fn init_db() -> Result<()> {
    // Determine a local path for the database, e.g., app data folder
    let mut db_path = std::env::current_exe()
        .unwrap_or_else(|_| PathBuf::from("."))
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .to_path_buf();
    
    db_path.push("ultranet_history.db");

    let conn = Connection::open(db_path)?;

    // Optimization history table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS optimization_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
            action_name TEXT NOT NULL,
            before_ping INTEGER,
            after_ping INTEGER,
            ping_diff INTEGER,
            summary TEXT
        )",
        [],
    )?;

    // General metrics tracking table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS metrics_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
            ping_ms INTEGER,
            jitter_ms INTEGER,
            loss_pct REAL,
            gateway_ping INTEGER,
            download_kbps REAL,
            upload_kbps REAL
        )",
        [],
    )?;

    // Adaptive Learning table for tracking incidents and optimizations
    conn.execute(
        "CREATE TABLE IF NOT EXISTS adaptive_learning (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
            game_name TEXT,
            ping_spike INTEGER,
            jitter INTEGER,
            wifi_signal INTEGER,
            action_taken TEXT,
            was_effective BOOLEAN,
            isp_routing_issue BOOLEAN
        )",
        [],
    )?;

    *DB_CONN.lock().unwrap() = Some(conn);
    tracing::info!("SQLite database initialized.");
    Ok(())
}

pub fn log_optimization(
    action_name: &str,
    before_ping: u32,
    after_ping: u32,
    ping_diff: i32,
    summary: &str,
) {
    if let Some(conn) = DB_CONN.lock().unwrap().as_ref() {
        let _ = conn.execute(
            "INSERT INTO optimization_history (action_name, before_ping, after_ping, ping_diff, summary)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![action_name, before_ping, after_ping, ping_diff, summary],
        ).map_err(|e| tracing::error!("Failed to log optimization to DB: {}", e));
    }
}

pub fn log_metrics(
    ping_ms: u32,
    jitter_ms: u32,
    loss_pct: f32,
    gateway_ping: u32,
    download_kbps: f32,
    upload_kbps: f32,
) {
    if let Some(conn) = DB_CONN.lock().unwrap().as_ref() {
        let _ = conn.execute(
            "INSERT INTO metrics_history (ping_ms, jitter_ms, loss_pct, gateway_ping, download_kbps, upload_kbps)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![ping_ms, jitter_ms, loss_pct, gateway_ping, download_kbps, upload_kbps],
        ).map_err(|e| tracing::error!("Failed to log metrics to DB: {}", e));
    }
}
pub fn log_adaptive_learning(
    game_name: &str,
    ping_spike: u32,
    jitter: u32,
    wifi_signal: u32,
    action_taken: &str,
    was_effective: bool,
    isp_routing_issue: bool,
) {
    if let Some(conn) = DB_CONN.lock().unwrap().as_ref() {
        let _ = conn.execute(
            "INSERT INTO adaptive_learning (game_name, ping_spike, jitter, wifi_signal, action_taken, was_effective, isp_routing_issue)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![game_name, ping_spike, jitter, wifi_signal, action_taken, was_effective, isp_routing_issue],
        ).map_err(|e| tracing::error!("Failed to log adaptive learning: {}", e));
    }
}
