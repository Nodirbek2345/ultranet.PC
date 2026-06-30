use std::process::Command;
use std::os::windows::process::CommandExt;

const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Detects default gateway IP from Windows routing table
pub fn get_default_gateway() -> String {
    // Use "route print 0.0.0.0" to find the default gateway
    if let Ok(output) = Command::new("cmd")
        .args(["/C", "route print 0.0.0.0"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
    {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            // Default route: 0.0.0.0  0.0.0.0  <gateway>  <interface>  <metric>
            if parts.len() >= 5 && parts[0] == "0.0.0.0" && parts[1] == "0.0.0.0" {
                let gw = parts[2].to_string();
                // Validate it looks like an IP
                if gw.contains('.') && gw != "0.0.0.0" {
                    return gw;
                }
            }
        }
    }

    // Fallback: try ipconfig
    if let Ok(output) = Command::new("ipconfig")
        .creation_flags(CREATE_NO_WINDOW)
        .output()
    {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            // English: "Default Gateway"
            // Russian: "Основной шлюз"
            if line.contains("Default Gateway") || line.contains("Основной шлюз") || line.contains("шлюз") {
                if let Some(ip_part) = line.split(':').nth(1) {
                    let ip = ip_part.trim().to_string();
                    if ip.contains('.') && !ip.is_empty() {
                        return ip;
                    }
                }
            }
        }
    }

    "192.168.1.1".to_string() // Last resort fallback
}

/// Get adapter speed in Mbps
pub fn get_adapter_speed() -> u32 {
    if let Ok(output) = Command::new("netsh")
        .args(["interface", "ipv4", "show", "subinterfaces"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
    {
        let text = String::from_utf8_lossy(&output.stdout);
        let mut max_speed: u32 = 0;
        for line in text.lines().skip(3) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            // Format: MTU  MediaSenseState  Bytes In  Bytes Out  Interface
            if parts.len() >= 4 {
                // Try to find link speed from separate command
            }
        }
        if max_speed > 0 { return max_speed; }
    }

    // Fallback via wmic
    if let Ok(output) = Command::new("cmd")
        .args(["/C", "wmic nic where \"NetEnabled=true\" get Speed /format:list"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
    {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            if line.starts_with("Speed=") {
                if let Ok(speed) = line.replace("Speed=", "").trim().parse::<u64>() {
                    return (speed / 1_000_000) as u32; // bits/s -> Mbps
                }
            }
        }
    }

    1000 // Default 1Gbps fallback
}

/// Get active TCP connections count
pub fn get_active_connections() -> u32 {
    if let Ok(output) = Command::new("cmd")
        .args(["/C", "netstat -n | find /c \"ESTABLISHED\""])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
    {
        let text = String::from_utf8_lossy(&output.stdout);
        return text.trim().parse().unwrap_or(0);
    }
    0
}
