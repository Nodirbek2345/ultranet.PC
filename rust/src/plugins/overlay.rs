use std::sync::{Arc, Mutex};
use std::thread;
use lazy_static::lazy_static;
use tracing::{info, warn};
use windows::{
    core::*, Win32::Foundation::*, Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::GetModuleHandleW, Win32::UI::WindowsAndMessaging::*,
};

lazy_static! {
    static ref OVERLAY_DATA: Arc<Mutex<OverlayData>> = Arc::new(Mutex::new(OverlayData {
        ping: 0,
        tcp: 0,
        gw: 0,
        jitter: 0,
        loss: 0.0,
        hwnd: HWND(0),
    }));
}

struct OverlayData {
    ping: u32,
    tcp: u32,
    gw: u32,
    jitter: u32,
    loss: f32,
    hwnd: HWND,
}

pub fn show_overlay() {
    let data = OVERLAY_DATA.lock().unwrap();
    if data.hwnd.0 != 0 { return; }
    
    info!("Starting Game Overlay...");
    
    thread::spawn(|| {
        unsafe {
            let instance = GetModuleHandleW(None).unwrap_or(HMODULE(0));
            let class_name = w!("UltraNetOverlayClass");

            let wc = WNDCLASSW {
                hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or(HCURSOR(0)),
                hInstance: instance.into(),
                lpszClassName: class_name,
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(wndproc),
                ..Default::default()
            };

            if RegisterClassW(&wc) == 0 {
                warn!("Failed to register overlay window class");
                return;
            }

            let hwnd = CreateWindowExW(
                WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
                class_name,
                w!("UltraNet Overlay"),
                WS_POPUP | WS_VISIBLE,
                50, 10, 800, 50, // X, Y, Width, Height
                None,
                None,
                instance,
                None,
            );

            if hwnd.0 == 0 {
                warn!("Failed to create overlay window");
                return;
            }

            // Set color key (black) as transparent
            let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0x00000000), 255, LWA_COLORKEY | LWA_ALPHA);

            {
                let mut data = OVERLAY_DATA.lock().unwrap();
                data.hwnd = hwnd;
            }

            let mut message = MSG::default();
            while GetMessageW(&mut message, None, 0, 0).into() {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
            
            // Clean up
            {
                let mut data = OVERLAY_DATA.lock().unwrap();
                data.hwnd = HWND(0);
            }
        }
    });
}

pub fn hide_overlay() {
    let hwnd = {
        let data = OVERLAY_DATA.lock().unwrap();
        data.hwnd
    };
    if hwnd.0 != 0 {
        unsafe {
            let _ = PostMessageW(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0));
        }
    }
}

pub fn update_overlay(ping: u32, tcp: u32, gw: u32, jitter: u32, loss: f32) {
    let hwnd = {
        let mut data = OVERLAY_DATA.lock().unwrap();
        if data.hwnd.0 == 0 { return; }
        data.ping = ping;
        data.tcp = tcp;
        data.gw = gw;
        data.jitter = jitter;
        data.loss = loss;
        data.hwnd
    };
    
    unsafe {
        InvalidateRect(hwnd, None, true);
    }
}

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match message {
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                let hdc = BeginPaint(window, &mut ps);
                
                // Clear background with black (which is color-keyed to transparent)
                let rect = RECT { left: 0, top: 0, right: 300, bottom: 100 };
                let hbrush = CreateSolidBrush(COLORREF(0x00000000));
                FillRect(hdc, &rect, hbrush);
                let _ = DeleteObject(hbrush);
                
                SetBkMode(hdc, TRANSPARENT);
                
                let (ping, tcp, gw, jitter, loss) = {
                    let data = OVERLAY_DATA.lock().unwrap();
                    (data.ping, data.tcp, data.gw, data.jitter, data.loss)
                };
                
                let text = format!("ICMP Ping: {}ms | TCP Ping: {}ms | Shlyuz: {}ms | Jitter: {}ms | Yo'qotish: {:.1}%", ping, tcp, gw, jitter, loss);
                let wtext: Vec<u16> = text.encode_utf16().collect();
                
                // Set Text Color based on ping
                if ping > 60 || loss > 1.0 {
                    SetTextColor(hdc, COLORREF(0x000000FF)); // Red
                } else if ping > 40 {
                    SetTextColor(hdc, COLORREF(0x0000FFFF)); // Yellow
                } else {
                    SetTextColor(hdc, COLORREF(0x0000FF00)); // Green
                }

                // Font
                let hfont = CreateFontW(
                    20, 0, 0, 0, FW_BOLD.0 as i32, 0, 0, 0, DEFAULT_CHARSET.0 as u32,
                    OUT_DEFAULT_PRECIS.0 as u32, CLIP_DEFAULT_PRECIS.0 as u32, DEFAULT_QUALITY.0 as u32, DEFAULT_PITCH.0 as u32,
                    w!("Consolas"),
                );
                SelectObject(hdc, hfont);

                TextOutW(hdc, 5, 5, &wtext);
                
                let _ = DeleteObject(hfont);
                EndPaint(window, &ps);
                LRESULT(0)
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(window, message, wparam, lparam),
        }
    }
}
