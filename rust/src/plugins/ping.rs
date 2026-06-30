use std::net::{IpAddr, SocketAddr};
use std::time::{Duration, Instant};
use surge_ping::{Client, Config, PingIdentifier, PingSequence, IcmpPacket};
use tokio::task;

#[derive(Clone, Debug)]
pub struct PingStats {
    pub target: String,
    pub ping_type: String,        // "ICMP", "TCP", "Gateway"
    pub average_ms: f32,
    pub best_ms: u32,
    pub worst_ms: u32,
    pub median_ms: u32,
    pub p95_ms: u32,
    pub jitter_ms: f32,
    pub packet_loss_pct: f32,
}

/// Run ICMP ping to multiple targets in parallel
pub async fn run_multi_ping(targets: Vec<String>, count: u16) -> Vec<PingStats> {
    let mut tasks = vec![];
    
    for target in targets {
        let t = target.clone();
        tasks.push(task::spawn(async move {
            ping_target_icmp(&t, count).await
        }));
    }

    let mut results = vec![];
    for t in tasks {
        if let Ok(Some(stats)) = t.await {
            results.push(stats);
        }
    }
    
    results
}

/// ICMP Ping implementation
async fn ping_target_icmp(target: &str, count: u16) -> Option<PingStats> {
    let addr: IpAddr = target.parse().ok()?;
    let config = Config::default();
    let client = Client::new(&config).ok()?;
    
    let mut pinger = client.pinger(addr, PingIdentifier(rand::random())).await;
    pinger.timeout(Duration::from_millis(1500));
    
    let payload = [0; 56];
    let mut history = vec![];
    
    for i in 0..count {
        match pinger.ping(PingSequence(i), &payload).await {
            Ok((IcmpPacket::V4(_), duration)) | Ok((IcmpPacket::V6(_), duration)) => {
                history.push(duration.as_millis() as u32);
            }
            Err(_) => {
                history.push(9999); // Drop
            }
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    
    calculate_stats(target, "ICMP", history)
}

/// TCP Ping (SYN latency to port 443)
pub async fn run_tcp_ping(target: &str, port: u16, count: u16) -> Option<PingStats> {
    let ip: IpAddr = target.parse().ok()?;
    let addr = SocketAddr::new(ip, port);
    
    let mut history = vec![];
    
    for _ in 0..count {
        let start = Instant::now();
        match tokio::time::timeout(
            Duration::from_millis(1500),
            tokio::net::TcpStream::connect(addr)
        ).await {
            Ok(Ok(_stream)) => {
                // TcpStream::connect completes a full 3-way handshake which takes ~2 RTTs.
                // We divide by 2 to approximate the actual single RTT latency.
                let mut rtt = (start.elapsed().as_millis() as u32) / 2;
                if rtt == 0 { rtt = 1; }
                history.push(rtt);
            }
            _ => {
                history.push(9999);
            }
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    
    calculate_stats(target, "TCP", history)
}

/// Gateway Ping (ICMP to detected gateway)
pub async fn run_gateway_ping(gateway_ip: &str, count: u16) -> Option<PingStats> {
    let result = ping_target_icmp(gateway_ip, count).await;
    result.map(|mut s| {
        s.ping_type = "Gateway".to_string();
        s
    })
}

fn calculate_stats(target: &str, ping_type: &str, history: Vec<u32>) -> Option<PingStats> {
    let total = history.len() as f32;
    if total == 0.0 { return None; }
    
    let lost = history.iter().filter(|&&p| p == 9999).count() as f32;
    let packet_loss_pct = (lost / total) * 100.0;
    
    let mut valid_pings: Vec<u32> = history.iter().filter(|&&p| p != 9999).copied().collect();
    if valid_pings.is_empty() {
        return Some(PingStats {
            target: target.to_string(),
            ping_type: ping_type.to_string(),
            average_ms: 999.0,
            best_ms: 999,
            worst_ms: 999,
            median_ms: 999,
            p95_ms: 999,
            jitter_ms: 0.0,
            packet_loss_pct: 100.0,
        });
    }
    
    valid_pings.sort_unstable();
    
    let best_ms = valid_pings[0];
    let worst_ms = valid_pings[valid_pings.len() - 1];
    let sum: u32 = valid_pings.iter().sum();
    let average_ms = sum as f32 / valid_pings.len() as f32;
    
    let mid = valid_pings.len() / 2;
    let median_ms = valid_pings[mid];
    
    let p95_idx = (valid_pings.len() as f32 * 0.95) as usize;
    let p95_ms = valid_pings[std::cmp::min(p95_idx, valid_pings.len() - 1)];
    
    // Jitter: average difference between consecutive pings
    let mut jitter_sum = 0;
    let mut diff_count = 0;
    let mut last = None;
    for &p in history.iter() {
        if p != 9999 {
            if let Some(prev) = last {
                jitter_sum += (p as i32 - prev as i32).abs();
                diff_count += 1;
            }
            last = Some(p);
        }
    }
    
    let jitter_ms = if diff_count > 0 { jitter_sum as f32 / diff_count as f32 } else { 0.0 };
    
    Some(PingStats {
        target: target.to_string(),
        ping_type: ping_type.to_string(),
        average_ms,
        best_ms,
        worst_ms,
        median_ms,
        p95_ms,
        jitter_ms,
        packet_loss_pct,
    })
}
