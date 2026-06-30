use std::process::Command;
use std::os::windows::process::CommandExt;
use anyhow::Result;
use tokio::task;

#[derive(Clone, Debug)]
pub struct HopStatus {
    pub hop: u8,
    pub ip: String,
    pub latency_ms: u32,
    pub is_timeout: bool,
}

#[derive(Clone, Debug)]
pub struct TracerouteResult {
    pub target: String,
    pub hops: Vec<HopStatus>,
    pub ai_conclusion: String,
}

const CREATE_NO_WINDOW: u32 = 0x08000000;

pub async fn run_traceroute_ai(target: String) -> Result<TracerouteResult> {
    // Run traceroute in a blocking task since it's a slow synchronous system command
    let t = target.clone();
    let hops = task::spawn_blocking(move || {
        let mut hops = vec![];
        let output = Command::new("tracert")
            .arg("-d")
            .arg("-h")
            .arg("20")
            .arg("-w")
            .arg("1000") // 1 sec timeout
            .arg(&t)
            .creation_flags(CREATE_NO_WINDOW)
            .output();

        if let Ok(out) = output {
            let text = String::from_utf8_lossy(&out.stdout);
            for line in text.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    if let Ok(hop) = parts[0].parse::<u8>() {
                        let is_timeout = parts.contains(&"*");
                        let mut latency_ms = 0;
                        let mut ip = String::new();
                        
                        if is_timeout {
                            ip = "Timeout".to_string();
                        } else {
                            // Example tracert line: 1    <1 ms    <1 ms    <1 ms  192.168.1.1
                            // Extract max latency
                            for part in &parts[1..parts.len()-1] {
                                let clean = part.replace("<", "").replace("ms", "");
                                if let Ok(val) = clean.parse::<u32>() {
                                    if val > latency_ms { latency_ms = val; }
                                }
                            }
                            ip = parts.last().unwrap().to_string();
                        }

                        hops.push(HopStatus {
                            hop,
                            ip,
                            latency_ms,
                            is_timeout,
                        });
                    }
                }
            }
        }
        hops
    }).await?;

    // Traceroute AI Logic
    let ai_conclusion = analyze_hops(&hops);

    Ok(TracerouteResult {
        target,
        hops,
        ai_conclusion,
    })
}

fn analyze_hops(hops: &[HopStatus]) -> String {
    if hops.is_empty() {
        return "Traceroute bajarilmadi.".to_string();
    }

    let first_hop = &hops[0];
    if first_hop.is_timeout || first_hop.latency_ms > 50 {
        return "Muammo sizning Wi-Fi yoki Lokal tarmog'ingizda (1-chi hop) boshlangan. Routerni tekshiring.".to_string();
    }

    let mut prev_latency = first_hop.latency_ms;
    for hop in hops.iter().skip(1) {
        if !hop.is_timeout && hop.latency_ms > prev_latency + 100 {
            if hop.hop <= 3 {
                return format!("Muammo Provayderingiz (ISP) tarmog'ida (Hop {}). Katta kechikish qo'shildi: {}ms", hop.hop, hop.latency_ms);
            } else {
                return format!("Muammo Xalqaro magistral yoki Server marshrutida (Hop {}). Kechikish: {}ms", hop.hop, hop.latency_ms);
            }
        }
        if !hop.is_timeout {
            prev_latency = hop.latency_ms;
        }
    }

    "Marshrut stabil. Maxsus sekinlashuv nuqtasi topilmadi.".to_string()
}
