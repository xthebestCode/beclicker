use windows::core::BOOL;
use windows::Win32::Foundation::{HWND, LPARAM};
use windows::Win32::UI::WindowsAndMessaging::{EnumWindows, GetWindowTextW, IsWindowVisible};

pub struct WindowManager;

impl WindowManager {
    /// Callback для EnumWindows — заполняет Vec<(title, hwnd_as_isize)>
    unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        unsafe {
            if IsWindowVisible(hwnd).as_bool() {
                let mut buf = [0u16; 512];
                let len = GetWindowTextW(hwnd, &mut buf);
                if len > 0 {
                    let title = String::from_utf16_lossy(&buf[..len as usize]);
                    let vec = &mut *(lparam.0 as *mut Vec<(String, isize)>);
                    vec.push((title, hwnd.0 as isize));
                }
            }
        }
        BOOL(1)
    }

    /// Возвращает список видимых окон как (title, hwnd_as_isize)
    pub fn get_windows_list() -> Vec<(String, isize)> {
        let mut list: Vec<(String, isize)> = Vec::new();
        unsafe {
            EnumWindows(Some(Self::enum_proc), LPARAM(&mut list as *mut _ as isize)).ok();
        }
        list
    }
}