use std::ffi::c_void;
use windows::Win32::Foundation::*;
use windows::Win32::NetworkManagement::WiFi::*;
use tracing::{info, warn};

#[derive(Clone, Debug)]
pub struct WifiStats {
    pub ssid: String,
    pub bssid: String,
    pub signal_pct: u32,
    pub channel: u16,
    pub frequency_ghz: f32,
    pub radio_type: String,
    pub authentication: String,
    pub receive_rate_mbps: u32,
    pub transmit_rate_mbps: u32,
    pub band: String,
    pub retry_count: u64,
    pub fcs_error_count: u64, // indicator of packet loss in PHY layer
    pub transmitted_frames: u64,
    pub received_frames: u64,
}

impl Default for WifiStats {
    fn default() -> Self {
        Self {
            ssid: "Unknown".to_string(),
            bssid: "00:00:00:00:00:00".to_string(),
            signal_pct: 0,
            channel: 0,
            frequency_ghz: 0.0,
            radio_type: "Unknown".to_string(),
            authentication: "Unknown".to_string(),
            receive_rate_mbps: 0,
            transmit_rate_mbps: 0,
            band: "Unknown".to_string(),
            retry_count: 0,
            fcs_error_count: 0,
            transmitted_frames: 0,
            received_frames: 0,
        }
    }
}

pub fn get_wifi_stats() -> Option<WifiStats> {
    unsafe {
        let mut handle: HANDLE = HANDLE(0);
        let mut negotiated_version = 0;
        let res = WlanOpenHandle(2, None, &mut negotiated_version, &mut handle);
        if res != 0 {
            return None;
        }

        let mut p_interface_list: *mut WLAN_INTERFACE_INFO_LIST = std::ptr::null_mut();
        let enum_res = WlanEnumInterfaces(handle, None, &mut p_interface_list);
        
        let mut stats = WifiStats::default();
        let mut success = false;

        if enum_res == 0 && !p_interface_list.is_null() {
            let list = &*p_interface_list;
            if list.dwNumberOfItems > 0 {
                let interface_info = list.InterfaceInfo[0];
                let guid = &interface_info.InterfaceGuid;
                
                // 1. Get Connection Attributes (Signal, Rx/Tx Rate)
                let mut data_size = 0;
                let mut p_data: *mut c_void = std::ptr::null_mut();
                let mut opcode_value_type = WLAN_OPCODE_VALUE_TYPE(0);
                
                if WlanQueryInterface(
                    handle,
                    guid,
                    wlan_intf_opcode_current_connection,
                    None,
                    &mut data_size,
                    &mut p_data,
                    Some(&mut opcode_value_type)
                ) == 0 && !p_data.is_null() {
                    let conn_attr = &*(p_data as *const WLAN_CONNECTION_ATTRIBUTES);
                    stats.signal_pct = conn_attr.wlanAssociationAttributes.wlanSignalQuality;
                    stats.receive_rate_mbps = conn_attr.wlanAssociationAttributes.ulRxRate / 1000;
                    stats.transmit_rate_mbps = conn_attr.wlanAssociationAttributes.ulTxRate / 1000;
                    WlanFreeMemory(p_data);
                    success = true;
                }

                // 2. Get Statistics (Retries, FCS Errors)
                let mut p_stats_data: *mut c_void = std::ptr::null_mut();
                if WlanQueryInterface(
                    handle,
                    guid,
                    wlan_intf_opcode_statistics,
                    None,
                    &mut data_size,
                    &mut p_stats_data,
                    Some(&mut opcode_value_type)
                ) == 0 && !p_stats_data.is_null() {
                    let wlan_stats = &*(p_stats_data as *const WLAN_STATISTICS);
                    stats.transmitted_frames = wlan_stats.MacUcastCounters.ullTransmittedFrameCount;
                    stats.received_frames = wlan_stats.MacUcastCounters.ullReceivedFrameCount;
                    
                    let phy_stats = &wlan_stats.PhyCounters[0];
                    stats.retry_count = phy_stats.ullRetryCount + phy_stats.ullMultipleRetryCount;
                    stats.fcs_error_count = phy_stats.ullFCSErrorCount;
                    
                    WlanFreeMemory(p_stats_data);
                }
            }
            WlanFreeMemory(p_interface_list as *mut c_void);
        }
        
        WlanCloseHandle(handle, None);
        
        if success {
            Some(stats)
        } else {
            None
        }
    }
}
