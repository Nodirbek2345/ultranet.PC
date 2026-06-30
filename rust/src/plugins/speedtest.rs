use std::time::Duration;
use reqwest::Client;
use tokio::task;

#[derive(Clone, Debug)]
pub struct BufferbloatResult {
    pub idle_ping_ms: u32,
    pub download_ping_ms: u32,
    pub upload_ping_ms: u32,
    pub grade: String,
}

pub async fn run_bufferbloat_test(target_ip: String, test_url: String) -> BufferbloatResult {
    // 1. Measure Idle Ping
    let idle_ping = measure_ping_avg(&target_ip, 5).await;

    // 2. Start Download in background and measure ping
    let client = Client::new();
    let url_clone = test_url.clone();
    
    let download_task = task::spawn(async move {
        // Just stream a large file for a few seconds
        let _ = client.get(&url_clone).send().await;
    });

    tokio::time::sleep(Duration::from_millis(500)).await;
    let download_ping = measure_ping_avg(&target_ip, 5).await;
    download_task.abort();

    // Calculate Grade
    let diff = (download_ping as i32 - idle_ping as i32).max(0) as u32;
    let grade = if diff < 5 {
        "A+"
    } else if diff < 15 {
        "A"
    } else if diff < 30 {
        "B"
    } else if diff < 60 {
        "C"
    } else if diff < 100 {
        "D"
    } else {
        "F"
    };

    BufferbloatResult {
        idle_ping_ms: idle_ping,
        download_ping_ms: download_ping,
        upload_ping_ms: 0, // Mock upload for now
        grade: grade.to_string(),
    }
}

async fn measure_ping_avg(target: &str, count: u16) -> u32 {
    if let Ok(addr) = target.parse::<std::net::IpAddr>() {
        if let Ok(client) = surge_ping::Client::new(&surge_ping::Config::default()) {
            let mut pinger = client.pinger(addr, surge_ping::PingIdentifier(rand::random())).await;
            pinger.timeout(Duration::from_millis(1000));
            let mut sum = 0;
            let mut valid = 0;
            for i in 0..count {
                if let Ok((_, dur)) = pinger.ping(surge_ping::PingSequence(i), &[0; 56]).await {
                    sum += dur.as_millis() as u32;
                    valid += 1;
                }
            }
            if valid > 0 { return sum / valid; }
        }
    }
    999
}
