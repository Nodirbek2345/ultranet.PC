use std::ffi::c_void;
use windows::Win32::Foundation::*;
use windows::Win32::NetworkManagement::WiFi::*;

pub fn get_native_wifi() {
    unsafe {
        let mut handle: HANDLE = HANDLE(0);
        let mut negotiated_version = 0;
        let res = WlanOpenHandle(2, None, &mut negotiated_version, &mut handle);
        if res == 0 {
            let mut p_interface_list: *mut WLAN_INTERFACE_INFO_LIST = std::ptr::null_mut();
            if WlanEnumInterfaces(handle, None, &mut p_interface_list) == 0 {
                if !p_interface_list.is_null() {
                    let list = &*p_interface_list;
                    if list.dwNumberOfItems > 0 {
                        let interface_info = list.InterfaceInfo[0];
                        let guid = &interface_info.InterfaceGuid;
                        
                        // Query Interface
                        let mut data_size = 0;
                        let mut p_data: *mut c_void = std::ptr::null_mut();
                        let mut opcode_value_type = WLAN_OPCODE_VALUE_TYPE(0);
                        
                        let query_res = WlanQueryInterface(
                            handle,
                            guid,
                            wlan_intf_opcode_statistics,
                            None,
                            &mut data_size,
                            &mut p_data,
                            Some(&mut opcode_value_type)
                        );
                        
                        if query_res == 0 && !p_data.is_null() {
                            let stats = &*(p_data as *const WLAN_STATISTICS);
                            println!("Transmitted: {}", stats.MacUcastCounters.ullTransmittedFrameCount);
                            // The PhyCounters is an array of WLAN_PHY_FRAME_STATISTICS
                            let phy_stats = &stats.PhyCounters[0];
                            println!("Retry count: {}", phy_stats.ullRetryCount);
                            println!("FCS Error count: {}", phy_stats.ullFCSErrorCount);
                            WlanFreeMemory(p_data);
                        }
                    }
                    WlanFreeMemory(p_interface_list as *mut c_void);
                }
            }
            WlanCloseHandle(handle, None);
        }
    }
}
