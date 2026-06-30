use rusqlite::{Connection, Result};
use std::sync::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    static ref DB_CONN: Mutex<Option<Connection>> = Mutex::new(None);
}

pub fn init_db(db_path: &str) -> Result<()> {
    let conn = Connection::open(db_path)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS network_history (
            id INTEGER PRIMARY KEY,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
            ping_ms INTEGER,
            jitter_ms INTEGER,
            packet_loss_pct REAL,
            download_speed REAL,
            upload_speed REAL,
            health_score INTEGER
        )",
        (),
    )?;
    
    // Auto-delete records older than 90 days to save space
    conn.execute(
        "DELETE FROM network_history WHERE timestamp <= datetime('now', '-90 days')",
        (),
    )?;

    *DB_CONN.lock().unwrap() = Some(conn);
    Ok(())
}

pub fn save_history(
    ping: u32,
    jitter: u32,
    loss: f32,
    download: f64,
    upload: f64,
    score: u32,
) -> Result<()> {
    let lock = DB_CONN.lock().unwrap();
    if let Some(conn) = lock.as_ref() {
        conn.execute(
            "INSERT INTO network_history (ping_ms, jitter_ms, packet_loss_pct, download_speed, upload_speed, health_score) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (ping, jitter, loss, download, upload, score),
        )?;
    }
    Ok(())
}

pub fn get_trend() -> String {
    let lock = DB_CONN.lock().unwrap();
    if let Some(conn) = lock.as_ref() {
        let mut stmt = conn.prepare("SELECT AVG(ping_ms) FROM network_history WHERE timestamp >= datetime('now', '-24 hours')").unwrap();
        let avg_24h: f64 = stmt.query_row((), |row| row.get(0)).unwrap_or(0.0);
        
        if avg_24h > 0.0 {
            return format!("Oxirgi 24 soatdagi o'rtacha Ping: {:.1}ms. Trend tahlili bo'yicha barqaror.", avg_24h);
        }
    }
    "Yetarli tarixiy ma'lumot yo'q".to_string()
}
