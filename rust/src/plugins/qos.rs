use std::os::windows::process::CommandExt;
use std::process::Command;
use tracing::{info, warn};
use windows::Win32::System::Threading::{
    OpenProcess, SetPriorityClass, PROCESS_SET_INFORMATION, HIGH_PRIORITY_CLASS,
    IDLE_PRIORITY_CLASS, NORMAL_PRIORITY_CLASS, PROCESS_CREATION_FLAGS,
};

const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn set_high_priority(pid: u32) {
    if let Err(e) = set_process_priority(pid, HIGH_PRIORITY_CLASS.0) {
        warn!("Failed to set high priority for {}: {}", pid, e);
    } else {
        info!("Set high priority for PID {}", pid);
    }
}

pub fn set_idle_priority(pid: u32) {
    if let Err(e) = set_process_priority(pid, IDLE_PRIORITY_CLASS.0) {
        warn!("Failed to set idle priority for {}: {}", pid, e);
    } else {
        info!("Set idle priority for PID {}", pid);
    }
}

pub fn set_normal_priority(pid: u32) {
    let _ = set_process_priority(pid, NORMAL_PRIORITY_CLASS.0);
}

fn set_process_priority(pid: u32, priority: u32) -> Result<(), String> {
    unsafe {
        let handle = OpenProcess(PROCESS_SET_INFORMATION, false, pid)
            .map_err(|e| format!("Failed to open process: {}", e))?;
        
        SetPriorityClass(handle, PROCESS_CREATION_FLAGS(priority))
            .map_err(|e| format!("Failed to set priority: {}", e))?;
    }
    Ok(())
}

pub fn limit_bandwidth(app_path: &str, app_name: &str) -> Result<(), String> {
    info!("Limiting bandwidth for {}", app_name);
    // Throttle rate is in Bytes/sec. 100000 = 100 KB/s
    let output = Command::new("netsh")
        .args([
            "pacer", "add", "rule", 
            &format!("name=Limit_{}", app_name),
            "dir=out",
            &format!("app={}", app_path),
            "throttle_rate=100000"
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output();
        
    match output {
        Ok(out) if out.status.success() => Ok(()),
        _ => Err(format!("Failed to add QoS rule for {}", app_name))
    }
}

pub fn remove_bandwidth_limit(app_name: &str) -> Result<(), String> {
    info!("Removing bandwidth limit for {}", app_name);
    let _ = Command::new("netsh")
        .args([
            "pacer", "delete", "rule", 
            &format!("name=Limit_{}", app_name)
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output();
    Ok(())
}
