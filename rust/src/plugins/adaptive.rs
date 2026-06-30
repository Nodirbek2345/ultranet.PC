use std::sync::{Arc, Mutex};
use std::time::Duration;
use lazy_static::lazy_static;
use sysinfo::System;
use tokio::time::sleep;
use tracing::{info, warn};

use crate::plugins::qos::{limit_bandwidth, remove_bandwidth_limit, set_high_priority, set_idle_priority};
use crate::plugins::ping::run_multi_ping;
use crate::plugins::extreme::{enable_extreme_mode, disable_extreme_mode};

lazy_static! {
    static ref ENGINE_ACTIVE: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
    static ref CURRENT_GAME: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
}

const GAMES: [&str; 7] = ["valorant.exe", "cs2.exe", "pubg.exe", "dota2.exe", "leagueoflegends.exe", "fortnite.exe", "gta5.exe"];
const BACKGROUND_APPS: [&str; 6] = ["steam.exe", "epicgameslauncher.exe", "onedrive.exe", "googledrivesync.exe", "dropbox.exe", "battle.net.exe"];

pub fn start_engine() {
    let mut active = ENGINE_ACTIVE.lock().unwrap();
    if *active { return; }
    *active = true;
    
    info!("Starting Adaptive Background Gaming Optimization Engine...");
    
    tokio::spawn(async move {
        let mut sys = System::new_all();
        let target_ip = "1.1.1.1"; // Cloudflare for ping testing
        
        loop {
            // Check if engine is still active
            {
                let active = ENGINE_ACTIVE.lock().unwrap();
                if !*active { break; }
            }
            
            sys.refresh_processes();
            
            // 1. Game Detection
            let mut found_game = None;
            for (pid, process) in sys.processes() {
                let name = process.name().to_lowercase();
                if GAMES.contains(&name.as_str()) {
                    found_game = Some((pid.as_u32(), name.clone()));
                    break;
                }
            }
            
            // State Change
            {
                let mut current = CURRENT_GAME.lock().unwrap();
                if current.as_deref() != found_game.as_ref().map(|(_, n)| n.as_str()) {
                    if let Some((pid, ref name)) = found_game {
                        info!("Game Detected: {} (PID {})", name, pid);
                        set_high_priority(pid);
                        // Trigger Background Optimization & Extreme Mode
                        optimize_background_apps(&sys);
                        enable_extreme_mode();
                        // Start overlay
                        crate::plugins::overlay::show_overlay();
                    } else if let Some(ref name) = *current {
                        info!("Game Closed: {}", name);
                        restore_background_apps();
                        disable_extreme_mode();
                        crate::plugins::overlay::hide_overlay();
                    }
                    *current = found_game.clone().map(|(_, n)| n);
                }
            }
            
            // 2. Adaptive Ping Engine
            if found_game.is_some() {
                let targets = vec![target_ip.to_string()];
                let gw_ip = crate::plugins::gateway::get_default_gateway();
                
                let (pings, tcp_result, gw_result) = tokio::join!(
                    run_multi_ping(targets, 2),
                    crate::plugins::ping::run_tcp_ping(&target_ip, 443, 2),
                    crate::plugins::ping::run_gateway_ping(&gw_ip, 2)
                );
                
                if let Some(ping) = pings.first() {
                    let p = ping.average_ms.round() as u32;
                    let j = ping.jitter_ms.round() as u32;
                    let l = ping.packet_loss_pct;
                    let tcp = tcp_result.map(|r| r.average_ms.round() as u32).unwrap_or(0);
                    let gw = gw_result.map(|r| r.average_ms.round() as u32).unwrap_or(0);
                    
                    crate::plugins::overlay::update_overlay(p, tcp, gw, j, l);
                    
                    if p > 40 || l > 1.0 || j > 10 {
                        tracing::warn!("High Latency Detected: Ping={}ms Jitter={}ms Loss={}%", p, j, l);
                    }
                }
                sleep(Duration::from_secs(2)).await;
            } else {
                sleep(Duration::from_secs(1)).await;
            }
        }
    });
}

pub fn stop_engine() {
    let mut active = ENGINE_ACTIVE.lock().unwrap();
    *active = false;
    restore_background_apps();
    disable_extreme_mode();
    crate::plugins::overlay::hide_overlay();
    info!("Adaptive Engine Stopped.");
}

fn optimize_background_apps(sys: &System) {
    info!("Running Live Optimization on background apps...");
    for (pid, process) in sys.processes() {
        let name = process.name().to_lowercase();
        if BACKGROUND_APPS.contains(&name.as_str()) {
            info!("Throttling background app: {}", name);
            set_idle_priority(pid.as_u32());
            let path = process.exe().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
            if !path.is_empty() {
                let _ = limit_bandwidth(&path, &name);
            }
        }
    }
}

fn restore_background_apps() {
    info!("Restoring network priority for background apps...");
    for app in BACKGROUND_APPS.iter() {
        let _ = remove_bandwidth_limit(app);
    }
}
