use sysinfo::System;

#[derive(Clone, Debug)]
pub struct ProcessTraffic {
    pub name: String,
    pub pid: u32,
    pub memory_mb: f64,
}

/// Known bandwidth-heavy processes
const HEAVY_PROCESSES: &[&str] = &[
    "steam", "steamwebhelper", "steamservice",
    "onedrive",
    "wuauclt", "usoclient", "musnotification",
    "discord",
    "chrome", "firefox", "msedge", "brave",
    "epicgameslauncher", "eadesktop",
    "dropbox", "googledrive",
    "qbittorrent", "utorrent", "bittorrent",
    "teams", "slack", "zoom",
    "svchost",
];

/// Detect top processes that may consume network bandwidth.
pub fn detect_background_traffic() -> Vec<ProcessTraffic> {
    let mut sys = System::new();
    sys.refresh_processes();

    let mut heavy: Vec<ProcessTraffic> = Vec::new();

    for (pid, process) in sys.processes() {
        let name_lower = process.name().to_lowercase();
        
        // Check if process matches known bandwidth-heavy patterns
        let is_heavy = HEAVY_PROCESSES.iter().any(|p| name_lower.contains(p));
        
        if is_heavy {
            let mem_mb = process.memory() as f64 / (1024.0 * 1024.0);
            
            // Only include processes using > 10 MB memory (likely active)
            if mem_mb > 10.0 {
                // Avoid duplicates
                if !heavy.iter().any(|h| h.name == name_lower) {
                    heavy.push(ProcessTraffic {
                        name: name_lower,
                        pid: pid.as_u32(),
                        memory_mb: (mem_mb * 10.0).round() / 10.0,
                    });
                }
            }
        }
    }

    // Sort by memory descending (as a proxy for activity)
    heavy.sort_by(|a, b| b.memory_mb.partial_cmp(&a.memory_mb).unwrap_or(std::cmp::Ordering::Equal));
    heavy.truncate(10); // Top 10
    heavy
}
