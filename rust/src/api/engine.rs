use flutter_rust_bridge::frb;
use std::sync::Mutex;
use lazy_static::lazy_static;
use windows::Win32::NetworkManagement::IpHelper::{GetIfTable2, FreeMibTable, MIB_IF_TABLE2};

use crate::plugins::ping::{run_multi_ping, run_tcp_ping, run_gateway_ping, PingStats};
use crate::plugins::wifi::{get_wifi_stats, WifiStats};
use crate::plugins::dns::{run_dns_benchmark, DnsBenchmark};
use crate::plugins::gateway::{get_default_gateway, get_adapter_speed, get_active_connections};
use crate::plugins::traffic::{detect_background_traffic, ProcessTraffic};
use crate::plugins::history::save_history;

#[derive(Clone, Debug)]
pub struct V2Metrics {
    pub pings: Vec<PingStats>,
    pub tcp_ping_ms: u32,
    pub gateway_ip: String,
    pub gateway_ping_ms: u32,
    pub wifi: Option<WifiStats>,
    pub dns: Vec<DnsBenchmark>,
    pub download_speed: f64,
    pub upload_speed: f64,
    pub health_score: u32,
    pub health_label: String,
    pub active_connections: u32,
    pub adapter_speed_mbps: u32,
    pub top_traffic: Vec<ProcessTraffic>,
}

lazy_static! {
    static ref LAST_RX: Mutex<u64> = Mutex::new(0);
    static ref LAST_TX: Mutex<u64> = Mutex::new(0);
    static ref CACHED_GATEWAY: Mutex<String> = Mutex::new(String::new());
    static ref AUTO_OPTIMIZE: Mutex<bool> = Mutex::new(false);
}

#[frb(sync)]
pub fn init_v2() {
    crate::logger::init_logger();
    if let Err(e) = crate::db::init_db() {
        tracing::error!("Failed to initialize SQLite DB: {}", e);
    }
    
    // Pre-detect gateway on init
    let gw = get_default_gateway();
    *CACHED_GATEWAY.lock().unwrap() = gw;

    std::thread::spawn(move || {
        if let Ok(rt) = tokio::runtime::Runtime::new() {
            rt.block_on(async {
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                    if *AUTO_OPTIMIZE.lock().unwrap() {
                        crate::plugins::optimizer::pro_auto_optimize_cycle().await;
                    }
                }
            });
        }
    });
}

#[frb(sync)]
pub fn enable_pro_auto_optimize() {
    *AUTO_OPTIMIZE.lock().unwrap() = true;
    tracing::info!("Pro Auto-Optimize Enabled");
}

#[frb(sync)]
pub fn disable_pro_auto_optimize() {
    *AUTO_OPTIMIZE.lock().unwrap() = false;
    tracing::info!("Pro Auto-Optimize Disabled");
}

#[frb(sync)]
pub fn start_adaptive_engine() {
    crate::plugins::stability_engine::start_stability_engine();
}

#[frb(sync)]
pub fn stop_adaptive_engine() {
    crate::plugins::stability_engine::stop_stability_engine();
}

#[frb(sync)]
pub fn get_about_info() -> String {
    "UltraNet AI\nDeveloper: Bekmurodov Nodirbek\nVersion: 1.0.0\nCopyright © Bekmurodov Nodirbek".to_string()
}

pub async fn get_v2_metrics() -> V2Metrics {
    // === Traffic Speed (Windows IP Helper) ===
    let mut rx_total: u64 = 0;
    let mut tx_total: u64 = 0;

    unsafe {
        let mut table_ptr: *mut MIB_IF_TABLE2 = std::ptr::null_mut();
        if GetIfTable2(&mut table_ptr).is_ok() {
            let table = &*table_ptr;
            let entries = std::slice::from_raw_parts(table.Table.as_ptr(), table.NumEntries as usize);
            for entry in entries {
                rx_total += entry.InOctets;
                tx_total += entry.OutOctets;
            }
            let _ = FreeMibTable(table_ptr as _);
        }
    }

    let mut dl_speed: f64 = 0.0;
    let mut ul_speed: f64 = 0.0;
    {
        let mut last_rx = LAST_RX.lock().unwrap();
        let mut last_tx = LAST_TX.lock().unwrap();

        if *last_rx > 0 && rx_total >= *last_rx {
            dl_speed = ((rx_total - *last_rx) as f64) / 2.0; // per second (2s interval)
        }
        if *last_tx > 0 && tx_total >= *last_tx {
            ul_speed = ((tx_total - *last_tx) as f64) / 2.0;
        }

        *last_rx = rx_total;
        *last_tx = tx_total;
    }

    // === Gateway ===
    let gateway_ip = CACHED_GATEWAY.lock().unwrap().clone();

    // === Parallel tasks ===
    let targets = vec![
        "1.1.1.1".to_string(), 
        "8.8.8.8".to_string(), 
        "9.9.9.9".to_string(),
    ];
    
    let gw_clone = gateway_ip.clone();
    
    // Run ICMP pings, TCP ping, gateway ping, DNS, WiFi, traffic all concurrently
    let (pings, tcp_result, gw_result, dns, wifi, active_conns, adapter_speed, traffic) = tokio::join!(
        run_multi_ping(targets, 4),
        run_tcp_ping("1.1.1.1", 443, 3),
        run_gateway_ping(&gw_clone, 3),
        run_dns_benchmark(),
        async { get_wifi_stats() },
        async { get_active_connections() },
        async { get_adapter_speed() },
        async { detect_background_traffic() },
    );

    let tcp_ping_ms = tcp_result.as_ref().map(|p| p.average_ms as u32).unwrap_or(0);
    let gateway_ping_ms = gw_result.as_ref().map(|p| p.average_ms as u32).unwrap_or(0);

    // === Ping stats ===
    let avg_ping = pings.first().map(|p| p.average_ms).unwrap_or(999.0);
    let avg_loss = pings.first().map(|p| p.packet_loss_pct).unwrap_or(100.0);
    let avg_jitter = pings.first().map(|p| p.jitter_ms).unwrap_or(0.0);
    let dns_fastest = dns.first().map(|d| d.latency_ms).unwrap_or(0);

    // === Health Score (Weighted Formula) ===
    let mut score = 100.0f32;

    // Ping (25%): < 80ms ideal, penalize above
    if avg_ping > 80.0 {
        let penalty = ((avg_ping - 80.0) / 10.0 * 2.0).min(25.0);
        score -= penalty;
    }

    // Jitter (20%): < 15ms ideal
    if avg_jitter > 15.0 {
        let penalty = ((avg_jitter - 15.0) / 5.0 * 3.0).min(20.0);
        score -= penalty;
    }

    // Packet Loss (25%): 0% ideal
    let loss_penalty = (avg_loss * 10.0).min(25.0);
    score -= loss_penalty;

    // DNS (5%): < 30ms ideal
    if dns_fastest > 30 {
        let penalty = ((dns_fastest - 30) as f32 / 10.0).min(5.0);
        score -= penalty;
    }

    // Signal (5%): > 80% ideal
    let signal = wifi.as_ref().map(|w| w.signal_pct).unwrap_or(100);
    if signal < 80 && signal > 0 {
        if signal < 50 { score -= 5.0; }
        else { score -= 2.0; }
    }

    // Gateway (10%): < 5ms ideal
    if gateway_ping_ms > 5 {
        let penalty = ((gateway_ping_ms - 5) as f32 / 5.0).min(10.0);
        score -= penalty;
    }

    // Background (10%): no heavy processes ideal
    let bg_penalty = (traffic.len() as f32 * 2.0).min(10.0);
    score -= bg_penalty;

    let health_score = score.max(0.0).min(100.0) as u32;

    let health_label = match health_score {
        90..=100 => "Excellent",
        75..=89 => "Good",
        50..=74 => "Average",
        25..=49 => "Poor",
        _ => "Critical",
    }.to_string();

    // === Save to History (SQLite) ===
    crate::db::log_metrics(
        avg_ping as u32,
        avg_jitter as u32,
        avg_loss,
        gateway_ping_ms,
        dl_speed as f32,
        ul_speed as f32,
    );

    V2Metrics {
        pings,
        tcp_ping_ms,
        gateway_ip,
        gateway_ping_ms,
        wifi,
        dns,
        download_speed: dl_speed,
        upload_speed: ul_speed,
        health_score,
        health_label,
        active_connections: active_conns,
        adapter_speed_mbps: adapter_speed,
        top_traffic: traffic,
    }
}
