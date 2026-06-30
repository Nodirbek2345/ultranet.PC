use windows::Win32::NetworkManagement::WiFi::*;
use windows::Win32::Foundation::*;

fn main() {
    let mut handle: HANDLE = HANDLE(0);
    let mut negotiated_version: u32 = 0;
    unsafe {
        let res = WlanOpenHandle(2, None, &mut negotiated_version, &mut handle);
        if res == 0 {
            println!("WlanOpenHandle success");
            WlanCloseHandle(handle, None);
        }
    }
}
