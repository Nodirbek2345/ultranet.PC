use serde::{Serialize, Deserialize};
use reqwest::Client;
use anyhow::Result;
use std::sync::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    static ref GEMINI_API_KEY: Mutex<String> = Mutex::new(String::new());
}

pub fn set_api_key(key: &str) {
    *GEMINI_API_KEY.lock().unwrap() = key.to_string();
}

#[derive(Serialize, Debug)]
pub struct NetworkPayload {
    pub icmp_ping: u32,
    pub tcp_ping: u32,
    pub gateway_ping: u32,
    pub jitter: u32,
    pub packet_loss: f32,
    pub dns_fastest_ms: u32,
    pub dns_fastest_server: String,
    pub wifi_signal: u8,
    pub wifi_channel: u16,
    pub wifi_band: String,
    pub gateway_ip: String,
    pub active_connections: u32,
    pub adapter_speed_mbps: u32,
    pub download_bps: f64,
    pub upload_bps: f64,
    pub background_apps: Vec<String>,
    pub traceroute_bottleneck: String,
}

/// Local AI Diagnostics — no API call needed
#[derive(Clone, Debug)]
pub struct DiagnosticItem {
    pub severity: String,      // "critical", "warning", "info", "ok"
    pub category: String,      // "Gateway", "WiFi", "DNS", "ISP", etc.
    pub problem: String,
    pub reason: String,
    pub how_checked: String,
    pub solution: String,
    pub expected_result: String,
    pub confidence_pct: u8,
}

pub fn run_local_diagnostics(payload: &NetworkPayload) -> Vec<DiagnosticItem> {
    let mut diagnostics = vec![];

    // 1. Gateway Ping Analysis
    if payload.gateway_ping > 10 {
        diagnostics.push(DiagnosticItem {
            severity: if payload.gateway_ping > 50 { "critical" } else { "warning" }.to_string(),
            category: "Gateway".to_string(),
            problem: format!("Gateway ping yuqori: {}ms", payload.gateway_ping),
            reason: "Router yuklanishi, Wi-Fi interferensiyasi yoki eski firmware.".to_string(),
            how_checked: format!("ICMP Ping {} ga yuborildi.", payload.gateway_ip),
            solution: "Routerni qayta yoqing. Firmware yangilang. Ethernet kabelga o'ting.".to_string(),
            expected_result: "Gateway ping < 5ms ga tushishi kerak.".to_string(),
            confidence_pct: 85,
        });
    } else {
        diagnostics.push(DiagnosticItem {
            severity: "ok".to_string(),
            category: "Gateway".to_string(),
            problem: format!("Gateway: {}ms — yaxshi.", payload.gateway_ping),
            reason: String::new(),
            how_checked: format!("ICMP Ping {}", payload.gateway_ip),
            solution: String::new(),
            expected_result: String::new(),
            confidence_pct: 95,
        });
    }

    // 2. Wi-Fi Signal Analysis
    if payload.wifi_signal > 0 && payload.wifi_signal < 50 {
        diagnostics.push(DiagnosticItem {
            severity: "critical".to_string(),
            category: "Wi-Fi".to_string(),
            problem: format!("Wi-Fi signal juda zaif: {}%", payload.wifi_signal),
            reason: "Routerdan uzoqlik, devorlar yoki boshqa qurilmalar interferensiyasi.".to_string(),
            how_checked: "Windows Native Wi-Fi API (netsh wlan show interfaces).".to_string(),
            solution: "Routerga yaqinlashing. 5GHz bandga o'ting. Ethernet kabeldan foydalaning.".to_string(),
            expected_result: "Signal 70%+ bo'lganda ping 20-40% kamayadi.".to_string(),
            confidence_pct: 90,
        });
    } else if payload.wifi_signal >= 50 && payload.wifi_signal < 75 {
        diagnostics.push(DiagnosticItem {
            severity: "warning".to_string(),
            category: "Wi-Fi".to_string(),
            problem: format!("Wi-Fi signal o'rtacha: {}%", payload.wifi_signal),
            reason: "Router va qurilma o'rtasidagi masofa yoki to'siqlar.".to_string(),
            how_checked: "Windows Native Wi-Fi API.".to_string(),
            solution: "5GHz bandga o'ting yoki routerga yaqinlashing.".to_string(),
            expected_result: "Signal 80%+ ga oshsa ping barqarorlashadi.".to_string(),
            confidence_pct: 75,
        });
    }

    // 3. 2.4GHz Band Warning
    if payload.wifi_band == "2.4 GHz" && payload.wifi_signal > 0 {
        diagnostics.push(DiagnosticItem {
            severity: "warning".to_string(),
            category: "Wi-Fi Band".to_string(),
            problem: "2.4 GHz bandda ulangansiz.".to_string(),
            reason: "2.4 GHz band ko'proq interferensiyaga ega va sekinroq.".to_string(),
            how_checked: format!("Wi-Fi Channel: {} (< 14 = 2.4GHz)", payload.wifi_channel),
            solution: "5 GHz tarmoqqa ulaning (odatda SSID_5G).".to_string(),
            expected_result: "5GHz da ping 10-30ms kamayishi mumkin.".to_string(),
            confidence_pct: 80,
        });
    }

    // 4. Packet Loss
    if payload.packet_loss > 0.5 {
        diagnostics.push(DiagnosticItem {
            severity: if payload.packet_loss > 3.0 { "critical" } else { "warning" }.to_string(),
            category: "Packet Loss".to_string(),
            problem: format!("Paketlar yo'qolmoqda: {:.1}%", payload.packet_loss),
            reason: "Wi-Fi interferensiyasi, kabel muammosi, yoki ISP nosozligi.".to_string(),
            how_checked: "ICMP Ping natijalaridan hisoblandi.".to_string(),
            solution: "Ethernet kabelga o'ting. Router va modemni qayta yoqing.".to_string(),
            expected_result: "Packet Loss 0% bo'lishi kerak.".to_string(),
            confidence_pct: 88,
        });
    }

    // 5. DNS Latency
    if payload.dns_fastest_ms > 50 {
        diagnostics.push(DiagnosticItem {
            severity: "warning".to_string(),
            category: "DNS".to_string(),
            problem: format!("DNS sekin: {}ms ({})", payload.dns_fastest_ms, payload.dns_fastest_server),
            reason: "ISP DNS serveri sekin yoki uzoqda joylashgan.".to_string(),
            how_checked: "DNS serverlariga ICMP Ping.".to_string(),
            solution: "DNS ni 1.1.1.1 (Cloudflare) yoki 8.8.8.8 (Google) ga o'zgartiring.".to_string(),
            expected_result: "DNS latency < 20ms ga tushadi.".to_string(),
            confidence_pct: 82,
        });
    }

    // 6. Background Traffic
    if !payload.background_apps.is_empty() {
        let apps = payload.background_apps.join(", ");
        diagnostics.push(DiagnosticItem {
            severity: "warning".to_string(),
            category: "Background Traffic".to_string(),
            problem: format!("Fon dasturlar internet ishlatyapti: {}", apps),
            reason: "Bu dasturlar tarmoq kanalini band qiladi va ping oshiradi.".to_string(),
            how_checked: "Windows Process Monitor orqali aniqlandi.".to_string(),
            solution: "O'yin yoki muhim ish vaqtida ushbu dasturlarni yoping.".to_string(),
            expected_result: "Ping 10-50ms ga kamayishi mumkin.".to_string(),
            confidence_pct: 70,
        });
    }

    // 7. High Ping but Gateway OK — ISP problem
    if payload.icmp_ping > 100 && payload.gateway_ping < 10 && payload.packet_loss < 1.0 {
        diagnostics.push(DiagnosticItem {
            severity: "info".to_string(),
            category: "ISP / Server".to_string(),
            problem: format!("Ping yuqori ({}ms) lekin Gateway yaxshi ({}ms).", payload.icmp_ping, payload.gateway_ping),
            reason: "Muammo sizning lokal tarmog'ingizda emas. ISP marshruti yoki server masofasi sababchi.".to_string(),
            how_checked: "Gateway vs Cloudflare ping taqqoslandi.".to_string(),
            solution: "ISP bilan bog'laning. VPN yoki routing optimizer (ExitLag) sinab ko'ring.".to_string(),
            expected_result: "Bu muammoni Windows optimallashtirish bilan hal qilib bo'lmaydi.".to_string(),
            confidence_pct: 92,
        });
    }

    // 8. TCP vs ICMP difference
    if payload.tcp_ping > 0 && payload.icmp_ping > 0 {
        let diff = (payload.tcp_ping as i32 - payload.icmp_ping as i32).abs();
        if diff > 30 {
            diagnostics.push(DiagnosticItem {
                severity: "warning".to_string(),
                category: "Firewall / QoS".to_string(),
                problem: format!("ICMP ({}ms) va TCP ({}ms) ping orasida katta farq.", payload.icmp_ping, payload.tcp_ping),
                reason: "Firewall yoki ISP QoS ICMP/TCP paketlarni turlicha ustuvorlashtirmoqda.".to_string(),
                how_checked: "ICMP Ping vs TCP Port 443 Connect latency taqqoslandi.".to_string(),
                solution: "Windows Firewall sozlamalarini tekshiring.".to_string(),
                expected_result: "Farq < 10ms bo'lishi normal.".to_string(),
                confidence_pct: 65,
            });
        }
    }

    diagnostics
}

// Gemini AI Response structs
#[derive(Deserialize, Debug)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Deserialize, Debug)]
struct Candidate {
    content: Content,
}

#[derive(Deserialize, Debug)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Deserialize, Debug)]
struct Part {
    text: String,
}

pub async fn get_ai_recommendation(payload: &NetworkPayload) -> Result<String> {
    let key = GEMINI_API_KEY.lock().unwrap().clone();
    if key.is_empty() {
        return Ok("API kaliti kiritilmagan".to_string());
    }

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent?key={}",
        key
    );

    let prompt = format!(
        "Siz Senior Network Engineersiz (Cloudflare/Cisco darajasida). Mijozning tarmoq holati:\n{}\n\n\
        Qat'iy quyidagi formatda javob bering:\n\
        MUAMMO: [Asosiy muammo]\n\
        SABAB: [Texnik sababi]\n\
        TEKSHIRISH: [Qanday aniqlandi]\n\
        YECHIM: [Aniq qadam-baqadam ko'rsatma]\n\
        KUTILGAN NATIJA: [Qanday yaxshilanishi kerak]\n\
        ISHONCHLILIK: [Masalan, 85%]\n\n\
        Agar muammo ISP yoki server tomonda bo'lsa, buni aniq ayting va yolg'on optimizatsiya va'da qilmang.",
        serde_json::to_string_pretty(payload).unwrap_or_default()
    );

    let request_body = serde_json::json!({
        "contents": [{"parts": [{"text": prompt}]}]
    });

    let client = Client::new();
    let res = client.post(&url)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;

    let text = res.text().await?;
    if let Ok(json_res) = serde_json::from_str::<GeminiResponse>(&text) {
        if let Some(candidate) = json_res.candidates.first() {
            if let Some(part) = candidate.content.parts.first() {
                return Ok(part.text.clone());
            }
        }
    }

    Ok("AI tahlilida xatolik yuz berdi.".to_string())
}
