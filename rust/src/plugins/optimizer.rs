use std::process::Command;
use std::os::windows::process::CommandExt;
use std::time::Duration;
use crate::plugins::ping::run_multi_ping;

const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Clone, Debug)]
pub struct OptimizationRecommendation {
    pub id: String,
    pub title: String,
    pub description: String,
    pub is_recommended: bool,
}

#[derive(Clone, Debug)]
pub struct OptimizationReport {
    pub before_ping: u32,
    pub after_ping: u32,
    pub ping_diff: i32,
    pub summary: String,
    pub actions_taken: Vec<String>,
}

pub async fn analyze_network() -> Vec<OptimizationRecommendation> {
    let mut recs = vec![];

    // 1. DNS Flush Recommendation
    recs.push(OptimizationRecommendation {
        id: "dns_flush".to_string(),
        title: "DNS Cache Flush".to_string(),
        description: "Clears old DNS records to resolve routing delays.".to_string(),
        is_recommended: true, // Generally safe and recommended
    });

    // 2. Winsock Reset Recommendation
    let (winsock_needed, tcp_needed) = analyze_tcp_winsock();
    recs.push(OptimizationRecommendation {
        id: "winsock_reset".to_string(),
        title: "Winsock Reset".to_string(),
        description: "Resets network socket configurations. Warning: May affect VPNs.".to_string(),
        is_recommended: winsock_needed,
    });

    // 3. TCP/IP Reset
    recs.push(OptimizationRecommendation {
        id: "tcp_ip_reset".to_string(),
        title: "TCP/IP Stack Reset".to_string(),
        description: "Resets IPv4 and IPv6 stack to defaults.".to_string(),
        is_recommended: tcp_needed,
    });

    // 4. Power Plan
    let needs_power = !is_high_performance();
    recs.push(OptimizationRecommendation {
        id: "power_plan".to_string(),
        title: "High Performance Power Plan".to_string(),
        description: "Ensures network adapter runs at maximum power.".to_string(),
        is_recommended: needs_power,
    });

    // 5. TCP Auto-Tuning
    recs.push(OptimizationRecommendation {
        id: "tcp_autotuning".to_string(),
        title: "TCP Auto-Tuning Optimization".to_string(),
        description: "Optimizes TCP receive window to maximize throughput and lower latency.".to_string(),
        is_recommended: true,
    });

    // 6. Network Throttling Index
    recs.push(OptimizationRecommendation {
        id: "network_throttling".to_string(),
        title: "Disable Network Throttling".to_string(),
        description: "Removes Windows multimedia network throttling for raw ping reduction.".to_string(),
        is_recommended: true,
    });

    recs
}

fn analyze_tcp_winsock() -> (bool, bool) {
    // Just a placeholder analysis. In a real system, we might check registry or recent errors.
    // We will only recommend if ping or jitter is exceptionally high, which we can't easily check synchronously here.
    // For now, default to false unless explicitly required.
    (false, false)
}

fn is_high_performance() -> bool {
    if let Ok(out) = Command::new("powercfg").arg("/getactivescheme").creation_flags(CREATE_NO_WINDOW).output() {
        let text = String::from_utf8_lossy(&out.stdout).to_lowercase();
        text.contains("high performance") || text.contains("ultimate performance") || text.contains("высокая производительность")
    } else {
        true
    }
}

pub async fn apply_optimization(actions: Vec<String>, before_ping: u32) -> OptimizationReport {
    let mut actions_taken = vec![];

    for action in actions {
        match action.as_str() {
            "dns_flush" => {
                if Command::new("ipconfig").arg("/flushdns").creation_flags(CREATE_NO_WINDOW).output().is_ok() {
                    actions_taken.push("DNS Cache Flush".to_string());
                    tracing::info!("Executed DNS Flush");
                }
            }
            "winsock_reset" => {
                if Command::new("netsh").args(["winsock", "reset"]).creation_flags(CREATE_NO_WINDOW).output().is_ok() {
                    actions_taken.push("Winsock Reset".to_string());
                    tracing::info!("Executed Winsock Reset");
                }
            }
            "tcp_ip_reset" => {
                if Command::new("netsh").args(["int", "ip", "reset"]).creation_flags(CREATE_NO_WINDOW).output().is_ok() {
                    actions_taken.push("TCP/IP Reset".to_string());
                    tracing::info!("Executed TCP/IP Reset");
                }
            }
            "power_plan" => {
                // Try to set high performance (GUID: 8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c)
                if Command::new("powercfg").args(["/setactive", "8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c"]).creation_flags(CREATE_NO_WINDOW).output().is_ok() {
                    actions_taken.push("Power Plan Set to High Performance".to_string());
                    tracing::info!("Executed Power Plan Optimization");
                }
            }
            "tcp_autotuning" => {
                if Command::new("netsh").args(["int", "tcp", "set", "global", "autotuninglevel=normal"]).creation_flags(CREATE_NO_WINDOW).output().is_ok() {
                    actions_taken.push("TCP Auto-Tuning Set to Normal".to_string());
                    tracing::info!("Executed TCP Auto-Tuning");
                }
            }
            "network_throttling" => {
                if Command::new("reg").args(["add", "HKLM\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Multimedia\\SystemProfile", "/v", "NetworkThrottlingIndex", "/t", "REG_DWORD", "/d", "0xffffffff", "/f"]).creation_flags(CREATE_NO_WINDOW).output().is_ok() {
                    actions_taken.push("Network Throttling Disabled".to_string());
                    tracing::info!("Executed Network Throttling Disable");
                }
            }
            _ => {}
        }
    }

    // Wait a moment for changes to take effect
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Retest Ping
    let after_stats = run_multi_ping(vec!["1.1.1.1".to_string()], 5).await;
    let after_ping = after_stats.first().map(|p| p.average_ms as u32).unwrap_or(before_ping);
    
    let ping_diff = after_ping as i32 - before_ping as i32;

    let summary = if ping_diff <= -3 {
        format!("Optimization successful. Ping reduced by {}ms.", ping_diff.abs())
    } else {
        "Optimization completed. No measurable improvement detected. Current bottleneck appears to be ISP routing, server distance, or wireless signal quality.".to_string()
    };

    // Log to DB
    crate::db::log_optimization(
        &actions_taken.join(", "),
        before_ping,
        after_ping,
        ping_diff,
        &summary,
    );

    OptimizationReport {
        before_ping,
        after_ping,
        ping_diff,
        summary,
        actions_taken,
    }
}

pub async fn pro_auto_optimize_cycle() {
    tracing::info!("Pro Auto-Optimize Cycle Started");
    let mut actions_taken = vec![];

    // 1. Throttle Background Apps Globally
    let mut sys = sysinfo::System::new_all();
    sys.refresh_processes();
    crate::plugins::stability_engine::optimize_background_apps(&sys, true);
    actions_taken.push("Background Apps Throttled".to_string());

    // 2. Check DNS and auto-fix if slow
    let dns_bench = crate::plugins::dns::run_dns_benchmark().await;
    let mut current_dns_latency = 999;
    if let Some(first) = dns_bench.first() {
        if first.target == "1.1.1.1" || first.target == "8.8.8.8" {
            // Already using fast DNS or it's the fastest
            current_dns_latency = first.latency_ms;
        } else {
            // Checking how slow the first configured is vs the fastest known
            current_dns_latency = first.latency_ms;
        }
    }

    if current_dns_latency > 50 {
        // Change DNS to 1.1.1.1 via PowerShell
        // "Get-NetAdapter | Where-Object {$_.Status -eq 'Up'} | Set-DnsClientServerAddress -ServerAddresses ('1.1.1.1','1.0.0.1')"
        tracing::info!("Auto-fixing DNS to Cloudflare (1.1.1.1)...");
        let ps_cmd = "Get-NetAdapter | Where-Object {$_.Status -eq 'Up'} | Set-DnsClientServerAddress -ServerAddresses ('1.1.1.1','1.0.0.1')";
        if Command::new("powershell").args(["-Command", ps_cmd]).creation_flags(CREATE_NO_WINDOW).output().is_ok() {
            actions_taken.push("DNS Switched to 1.1.1.1".to_string());
        }
    }

    // 3. Auto Flush DNS if ping is unstable
    let pings = run_multi_ping(vec!["1.1.1.1".to_string()], 2).await;
    if let Some(ping) = pings.first() {
        if ping.average_ms > 60.0 || ping.packet_loss_pct > 0.0 {
            if Command::new("ipconfig").arg("/flushdns").creation_flags(CREATE_NO_WINDOW).output().is_ok() {
                actions_taken.push("DNS Cache Flushed".to_string());
            }
        }
    }

    // 4. Pro Local Network & Wi-Fi Optimization
    // Check Wi-Fi signal
    if let Some(wifi) = crate::plugins::wifi::get_wifi_stats() {
        if wifi.signal_pct < 50 && wifi.signal_pct > 0 {
            tracing::info!("Wi-Fi signal is critically low ({}%). Forcing rescan/reconnect...", wifi.signal_pct);
            // Reconnect to current profile to maybe jump to a better AP or 5GHz band
            if Command::new("netsh").args(["wlan", "disconnect"]).creation_flags(CREATE_NO_WINDOW).output().is_ok() {
                tokio::time::sleep(Duration::from_millis(500)).await;
                // Note: wifi.ssid might need escaping if it has spaces, but netsh usually handles name="ssid"
                let ssid_arg = format!("name=\"{}\"", wifi.ssid);
                let _ = Command::new("netsh").args(["wlan", "connect", &ssid_arg]).creation_flags(CREATE_NO_WINDOW).output();
                actions_taken.push(format!("Wi-Fi Reconnected (Signal was {}%)", wifi.signal_pct));
            }
        }
    }

    // 5. Check local gateway and clear ARP / Renew IP if needed
    let gw_ip = crate::plugins::gateway::get_default_gateway();
    if !gw_ip.is_empty() {
        let gw_ping = crate::plugins::ping::run_gateway_ping(&gw_ip, 2).await;
        if let Some(p) = gw_ping {
            if p.average_ms > 20.0 || p.packet_loss_pct > 0.0 {
                tracing::info!("Gateway latency is high ({}ms). Clearing ARP cache...", p.average_ms);
                // Clear ARP using PowerShell
                let ps_cmd = "Remove-NetNeighbor -AddressFamily IPv4 -Confirm:$false";
                let _ = Command::new("powershell").args(["-Command", ps_cmd]).creation_flags(CREATE_NO_WINDOW).output();
                actions_taken.push("ARP Cache Cleared".to_string());
                
                if p.average_ms > 100.0 {
                    // Extreme measure: Release / Renew
                    tracing::info!("Gateway latency is critical ({}ms). Renewing DHCP IP...", p.average_ms);
                    let _ = Command::new("ipconfig").arg("/release").creation_flags(CREATE_NO_WINDOW).output();
                    let _ = Command::new("ipconfig").arg("/renew").creation_flags(CREATE_NO_WINDOW).output();
                    actions_taken.push("DHCP IP Renewed".to_string());
                }
            }
        }
    }

    if !actions_taken.is_empty() {
        tracing::info!("Pro Auto-Optimize applied: {:?}", actions_taken);
    }
}
