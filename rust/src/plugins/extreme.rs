use std::os::windows::process::CommandExt;
use std::process::Command;
use tracing::{info, warn};

const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn enable_extreme_mode() {
    info!("Enabling Extreme Gaming Optimization Mode...");
    
    // 1. Get active interface GUID
    let active_guid = get_active_interface_guid();
    
    // 2. Disable Nagle's Algorithm for the active interface
    if let Some(guid) = &active_guid {
        let base_path = format!("HKLM\\SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters\\Interfaces\\{}", guid);
        
        let _ = Command::new("reg")
            .args(["add", &base_path, "/v", "TcpAckFrequency", "/t", "REG_DWORD", "/d", "1", "/f"])
            .creation_flags(CREATE_NO_WINDOW).output();
            
        let _ = Command::new("reg")
            .args(["add", &base_path, "/v", "TCPNoDelay", "/t", "REG_DWORD", "/d", "1", "/f"])
            .creation_flags(CREATE_NO_WINDOW).output();
            
        info!("Disabled Nagle's Algorithm on {}", guid);
    }

    // 3. System Responsiveness = 0
    let _ = Command::new("reg")
        .args(["add", "HKLM\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Multimedia\\SystemProfile", "/v", "SystemResponsiveness", "/t", "REG_DWORD", "/d", "0", "/f"])
        .creation_flags(CREATE_NO_WINDOW).output();
        
    // 4. Stop Background Services
    let _ = Command::new("net").args(["stop", "wuauserv"]).creation_flags(CREATE_NO_WINDOW).output();
    let _ = Command::new("net").args(["stop", "bits"]).creation_flags(CREATE_NO_WINDOW).output();
    
    // 5. Flush DNS
    let _ = Command::new("ipconfig").args(["/flushdns"]).creation_flags(CREATE_NO_WINDOW).output();
}

pub fn disable_extreme_mode() {
    info!("Disabling Extreme Gaming Optimization Mode...");
    
    let active_guid = get_active_interface_guid();
    
    // 1. Revert Nagle's
    if let Some(guid) = &active_guid {
        let base_path = format!("HKLM\\SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters\\Interfaces\\{}", guid);
        
        // Delete the keys to restore default behavior
        let _ = Command::new("reg")
            .args(["delete", &base_path, "/v", "TcpAckFrequency", "/f"])
            .creation_flags(CREATE_NO_WINDOW).output();
            
        let _ = Command::new("reg")
            .args(["delete", &base_path, "/v", "TCPNoDelay", "/f"])
            .creation_flags(CREATE_NO_WINDOW).output();
    }

    // 2. Revert System Responsiveness (Default is 20)
    let _ = Command::new("reg")
        .args(["add", "HKLM\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Multimedia\\SystemProfile", "/v", "SystemResponsiveness", "/t", "REG_DWORD", "/d", "20", "/f"])
        .creation_flags(CREATE_NO_WINDOW).output();
        
    // 3. Start Background Services
    let _ = Command::new("net").args(["start", "wuauserv"]).creation_flags(CREATE_NO_WINDOW).output();
    let _ = Command::new("net").args(["start", "bits"]).creation_flags(CREATE_NO_WINDOW).output();
}

fn get_active_interface_guid() -> Option<String> {
    let output = Command::new("wmic")
        .args(["nicconfig", "where", "IPEnabled=True", "get", "SettingID"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;
        
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('{') && trimmed.ends_with('}') {
            return Some(trimmed.to_string());
        }
    }
    None
}
