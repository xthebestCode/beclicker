use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use parking_lot::RwLock;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{PostMessageW, WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDOWN, WM_LBUTTONUP};

pub struct Clicker;

impl Clicker {
    pub fn start_clicker(
        running: Arc<AtomicBool>,
        selected_hwnd: Arc<RwLock<Option<isize>>>,
        interval_ms: Arc<RwLock<u64>>,
        hold_shift: Arc<AtomicBool>,
        hold_ctrl: Arc<AtomicBool>,
    ) {
        std::thread::spawn(move || {
            let mut was_shift_held = false;
            let mut was_ctrl_held = false;

            loop {
                if running.load(Ordering::SeqCst) {
                    if let Some(hwnd_val) = *selected_hwnd.read() {
                        let hwnd = HWND(hwnd_val as *mut _);

                        let hold_shift_val = hold_shift.load(Ordering::SeqCst);
                        let hold_ctrl_val = hold_ctrl.load(Ordering::SeqCst);
                        println!("KEY: WPARAM(0x{:X}) LPARAM(0x{:X})", WPARAM(0).0, LPARAM(0).0);

                        if hold_shift_val != was_shift_held {
                            if hold_shift_val {
                                unsafe {
                                    PostMessageW(Some(hwnd), WM_KEYDOWN, WPARAM(0x10usize), LPARAM(0x001D0001)).ok();
                                    PostMessageW(Some(hwnd), WM_KEYDOWN, WPARAM(0xA0usize), LPARAM(0x001D0001)).ok();
                                    PostMessageW(Some(hwnd), WM_KEYDOWN, WPARAM(0xA1usize), LPARAM(0x001D0001)).ok();
                                }
                            } else {
                                unsafe {
                                    PostMessageW(Some(hwnd), WM_KEYUP, WPARAM(0x10usize), LPARAM(0xC02A0001)).ok();
                                }
                            }
                            was_shift_held = hold_shift_val;
                        }

                        if hold_ctrl_val != was_ctrl_held {
                            if hold_ctrl_val {
                                unsafe {
                                    PostMessageW(Some(hwnd), WM_KEYDOWN, WPARAM(0xA2 as usize), LPARAM(0x001D0001)).ok();
                                }
                            } else {
                                unsafe {
                                    PostMessageW(Some(hwnd), WM_KEYUP, WPARAM(0xA2 as usize), LPARAM(0xC01D0001)).ok();
                                }
                            }
                            was_ctrl_held = hold_ctrl_val;
                        }

                        unsafe {
                            PostMessageW(Some(hwnd), WM_LBUTTONDOWN, WPARAM(1), LPARAM(0)).ok();
                            PostMessageW(Some(hwnd), WM_LBUTTONUP, WPARAM(0), LPARAM(0)).ok();
                        }
                    }
                    let ms = *interval_ms.read();
                    std::thread::sleep(std::time::Duration::from_millis(ms));
                } else {
                    if was_shift_held {
                        if let Some(hwnd_val) = *selected_hwnd.read() {
                            let hwnd = HWND(hwnd_val as *mut _);
                            unsafe {
                                PostMessageW(Some(hwnd), WM_KEYUP, WPARAM(0x10 as usize), LPARAM(0xC02A0001)).ok();
                            }
                        }
                        was_shift_held = false;
                    }

                    if was_ctrl_held {
                        if let Some(hwnd_val) = *selected_hwnd.read() {
                            let hwnd = HWND(hwnd_val as *mut _);
                            unsafe {
                                PostMessageW(Some(hwnd), WM_KEYUP, WPARAM(0xA2 as usize), LPARAM(0xC01D0001)).ok();
                            }
                        }
                        was_ctrl_held = false;
                    }

                    std::thread::sleep(std::time::Duration::from_millis(60));
                }
            }
        });
    }
}