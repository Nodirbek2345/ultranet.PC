use std::sync::{Arc, Mutex};
use std::time::Duration;
use lazy_static::lazy_static;
use sysinfo::System;
use tokio::time::sleep;
use tracing::{info, warn};

use crate::plugins::qos::{limit_bandwidth, remove_bandwidth_limit, set_high_priority, set_idle_priority, set_normal_priority};
use crate::plugins::ping::run_multi_ping;
use crate::plugins::wifi::get_wifi_stats;
use crate::db::log_adaptive_learning;

lazy_static! {
    static ref ENGINE_ACTIVE: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
    static ref CURRENT_GAME: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    static ref JITTER_TREND: Arc<Mutex<Vec<u32>>> = Arc::new(Mutex::new(Vec::new()));
}

const GAMES: [&str; 7] = ["valorant.exe", "cs2.exe", "pubg.exe", "dota2.exe", "leagueoflegends.exe", "fortnite.exe", "gta5.exe"];
const BACKGROUND_APPS: [&str; 11] = [
    "steam.exe", "epicgameslauncher.exe", "onedrive.exe", "googledrivesync.exe", 
    "dropbox.exe", "battle.net.exe", "chrome.exe", "msedge.exe", "firefox.exe", "opera.exe", "svchost.exe" // svchost used carefully
];

pub fn start_stability_engine() {
    let mut active = ENGINE_ACTIVE.lock().unwrap();
    if *active { return; }
    *active = true;
    
    info!("Starting UltraNet AI - Adaptive Signal Stability Engine...");
    
    // Spawn a standard OS thread that runs a Tokio runtime
    std::thread::spawn(move || {
        if let Ok(rt) = tokio::runtime::Runtime::new() {
            rt.block_on(async {
                // 1-second Wi-Fi Monitoring Loop
                let wifi_loop = tokio::spawn(async move {
                    loop {
                        {
                            if !*ENGINE_ACTIVE.lock().unwrap() { break; }
                        }
                        if let Some(_stats) = get_wifi_stats() {
                            // Monitoring Wi-Fi stats
                        }
                        sleep(Duration::from_secs(1)).await;
                    }
                });

                // 2-second Ping & Game Loop
                let ping_loop = tokio::spawn(async move {
                    let mut sys = System::new_all();
                    let target_ip = "1.1.1.1"; 
                    
                    loop {
                        {
                            if !*ENGINE_ACTIVE.lock().unwrap() { break; }
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
                                    info!("UltraNet AI detected Game: {} (PID {})", name, pid);
                                    set_high_priority(pid);
                                    // Trigger initial soft optimization
                                    optimize_background_apps(&sys, false); 
                                    crate::plugins::overlay::show_overlay();
                                } else if let Some(ref name) = *current {
                                    info!("Game Closed: {}", name);
                                    restore_background_apps();
                                    crate::plugins::overlay::hide_overlay();
                                }
                                *current = found_game.clone().map(|(_, n)| n);
                            }
                        }
                        
                        // 2. Anti-Jitter Engine & Smart Stability
                        if let Some((_, ref game_name)) = found_game {
                            let pings = run_multi_ping(vec![target_ip.to_string()], 2).await;
                            let wifi = get_wifi_stats();
                            
                            if let Some(ping) = pings.first() {
                                let p = ping.average_ms.round() as u32;
                                let j = ping.jitter_ms.round() as u32;
                                let l = ping.packet_loss_pct;
                                
                                crate::plugins::overlay::update_overlay(p, 0, 0, j, l);
                                
                                // Maintain Jitter trend (60 seconds = 30 ticks of 2s)
                                {
                                    let mut trend = JITTER_TREND.lock().unwrap();
                                    trend.push(j);
                                    if trend.len() > 30 {
                                        trend.remove(0);
                                    }
                                }
                                
                                if j > 10 || l > 1.0 || p > 60 {
                                    let wifi_sig = wifi.as_ref().map(|w| w.signal_pct).unwrap_or(0);
                                    
                                    // Adaptive Anti-Jitter logic
                                    let is_wifi_good = wifi_sig > 70;
                                    let mut action = String::new();
                                    let mut isp_issue = false;

                                    if is_wifi_good {
                                        warn!("Ping spike detected while Wi-Fi is good. Throttling all background apps.");
                                        optimize_background_apps(&sys, true);
                                        action = "Aggressive Background Throttling".to_string();
                                    } else {
                                        warn!("Ping spike due to Poor Wi-Fi Signal ({}%).", wifi_sig);
                                        action = "Notify User: Poor Wi-Fi".to_string();
                                        isp_issue = false;
                                    }

                                    // Store in Adaptive Learning DB
                                    log_adaptive_learning(
                                        game_name,
                                        p,
                                        j,
                                        wifi_sig,
                                        &action,
                                        false, 
                                        isp_issue
                                    );
                                }
                            }
                            sleep(Duration::from_secs(2)).await;
                        } else {
                            sleep(Duration::from_secs(5)).await;
                        }
                    }
                });

                let _ = tokio::join!(wifi_loop, ping_loop);
            });
        }
    });
}

pub fn stop_stability_engine() {
    let mut active = ENGINE_ACTIVE.lock().unwrap();
    *active = false;
    restore_background_apps();
    crate::plugins::overlay::hide_overlay();
    info!("UltraNet AI - Stability Engine Stopped.");
}

pub fn optimize_background_apps(sys: &System, aggressive: bool) {
    info!("UltraNet AI: Running Background Optimizer...");
    for (pid, process) in sys.processes() {
        let name = process.name().to_lowercase();
        if BACKGROUND_APPS.contains(&name.as_str()) {
            // svchost requires care, don't bandwidth throttle all of it unless aggressive
            if name == "svchost.exe" && !aggressive { continue; }
            
            set_idle_priority(pid.as_u32());
            let path = process.exe().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
            if !path.is_empty() {
                // Ignore errors here if it fails to add rule
                let _ = limit_bandwidth(&path, &name);
            }
        }
    }
}

fn restore_background_apps() {
    info!("UltraNet AI: Restoring network priority...");
    for app in BACKGROUND_APPS.iter() {
        let _ = remove_bandwidth_limit(app);
    }
}
