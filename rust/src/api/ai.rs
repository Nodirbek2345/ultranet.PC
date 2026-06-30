use flutter_rust_bridge::frb;
use crate::plugins::ai::{get_ai_recommendation, run_local_diagnostics, NetworkPayload, DiagnosticItem};
use crate::plugins::traceroute::{run_traceroute_ai, TracerouteResult};
use crate::plugins::speedtest::{run_bufferbloat_test, BufferbloatResult};


#[frb(sync)]
pub fn set_api_key(key: String) {
    crate::plugins::ai::set_api_key(&key);
}

/// Run local AI diagnostics (no API call — instant)
pub async fn get_diagnostics(
    icmp_ping: u32,
    tcp_ping: u32,
    gateway_ping: u32,
    gateway_ip: String,
    jitter: u32,
    packet_loss: f32,
    dns_ms: u32,
    dns_server: String,
    wifi_signal: u8,
    wifi_channel: u16,
    wifi_band: String,
    active_connections: u32,
    adapter_speed: u32,
    download: f64,
    upload: f64,
    background_apps: Vec<String>,
) -> Vec<DiagnosticItem> {
    let payload = NetworkPayload {
        icmp_ping,
        tcp_ping,
        gateway_ping,
        jitter,
        packet_loss,
        dns_fastest_ms: dns_ms,
        dns_fastest_server: dns_server,
        wifi_signal,
        wifi_channel,
        wifi_band,
        gateway_ip,
        active_connections,
        adapter_speed_mbps: adapter_speed,
        download_bps: download,
        upload_bps: upload,
        background_apps,
        traceroute_bottleneck: String::new(),
    };
    
    run_local_diagnostics(&payload)
}

/// Get Gemini AI advice with full payload
pub async fn get_ai_advice_v2(
    icmp_ping: u32,
    tcp_ping: u32,
    gateway_ping: u32,
    gateway_ip: String,
    jitter: u32,
    packet_loss: f32,
    dns_ms: u32,
    dns_server: String,
    wifi_signal: u8,
    wifi_channel: u16,
    wifi_band: String,
    active_connections: u32,
    adapter_speed: u32,
    download: f64,
    upload: f64,
    background_apps: Vec<String>,
) -> String {
    let payload = NetworkPayload {
        icmp_ping,
        tcp_ping,
        gateway_ping,
        jitter,
        packet_loss,
        dns_fastest_ms: dns_ms,
        dns_fastest_server: dns_server,
        wifi_signal,
        wifi_channel,
        wifi_band,
        gateway_ip,
        active_connections,
        adapter_speed_mbps: adapter_speed,
        download_bps: download,
        upload_bps: upload,
        background_apps,
        traceroute_bottleneck: String::new(),
    };
    
    get_ai_recommendation(&payload).await.unwrap_or_else(|e| e.to_string())
}

pub async fn run_traceroute(target: String) -> TracerouteResult {
    run_traceroute_ai(target).await.unwrap_or(TracerouteResult {
        target: "".to_string(),
        hops: vec![],
        ai_conclusion: "Xatolik yuz berdi".to_string(),
    })
}

pub async fn run_bufferbloat(target: String, url: String) -> BufferbloatResult {
    run_bufferbloat_test(target, url).await
}

use crate::plugins::optimizer::{analyze_network as optimizer_analyze_network, apply_optimization as optimizer_apply_optimization, OptimizationRecommendation, OptimizationReport};

pub async fn analyze_network() -> Vec<OptimizationRecommendation> {
    optimizer_analyze_network().await
}

pub async fn apply_optimization(actions: Vec<String>, before_ping: u32) -> OptimizationReport {
    optimizer_apply_optimization(actions, before_ping).await
}
