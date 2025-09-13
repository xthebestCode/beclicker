use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use parking_lot::RwLock;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{PostMessageW, WM_LBUTTONDOWN, WM_LBUTTONUP};

pub struct Clicker;

impl Clicker {
    pub fn start_clicker(
        running: Arc<AtomicBool>,
        selected_hwnd: Arc<RwLock<Option<isize>>>,
        interval_ms: Arc<RwLock<u64>>,
    ) {
        std::thread::spawn(move || {
            loop {
                if running.load(Ordering::SeqCst) {
                    if let Some(hwnd_val) = *selected_hwnd.read() {
                        let hwnd = HWND(hwnd_val as *mut _);
                        unsafe {
                            PostMessageW(Some(hwnd), WM_LBUTTONDOWN, WPARAM(1), LPARAM(0)).ok();
                            PostMessageW(Some(hwnd), WM_LBUTTONUP, WPARAM(0), LPARAM(0)).ok();
                        }
                    }
                    let ms = *interval_ms.read();
                    std::thread::sleep(std::time::Duration::from_millis(ms));
                } else {
                    std::thread::sleep(std::time::Duration::from_millis(60));
                }
            }
        });
    }
}