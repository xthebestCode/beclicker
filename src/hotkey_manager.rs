use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use parking_lot::RwLock;
use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;

pub struct HotkeyManager;

impl HotkeyManager {
    /// Вспомогательная таблица: метка -> виртуальный key code (VK)
    pub fn vk_for_label(label: &str) -> u32 {
        match label {
            "F6" => 0x75,
            "F7" => 0x76,
            "F8" => 0x77,
            "F9" => 0x78,
            "F10" => 0x79,
            "R" => 0x52,
            "PgDn" => 0x22,
            _ => 0x75,
        }
    }

    pub const VK_LSHIFT: u32 = 0xA0;
    pub const VK_RSHIFT: u32 = 0xA1;
    pub const VK_SHIFT: u32 = 0x10;

    pub fn label_for_vk(vk: u32) -> String {
        match vk {
            0x75 => "F6".to_string(),
            0x76 => "F7".to_string(),
            0x77 => "F8".to_string(),
            0x78 => "F9".to_string(),
            0x79 => "F10".to_string(),
            0x52 => "R".to_string(),
            0x22 => "PgDn".to_string(),
            other => format!("VK_{:#X}", other),
        }
    }

    pub fn vk_to_key_name(vk: u32) -> String {
        match vk {
            0x08 => "Backspace".to_string(),
            0x09 => "Tab".to_string(),
            0x0D => "Enter".to_string(),
            0x10 => "Shift".to_string(),
            0x11 => "Ctrl".to_string(),
            0x12 => "Alt".to_string(),
            0x13 => "Pause".to_string(),
            0x14 => "Caps Lock".to_string(),
            0x1B => "Esc".to_string(),
            0x20 => "Space".to_string(),
            0x21 => "Page Up".to_string(),
            0x22 => "Page Down".to_string(),
            0x23 => "End".to_string(),
            0x24 => "Home".to_string(),
            0x25 => "Left Arrow".to_string(),
            0x26 => "Up Arrow".to_string(),
            0x27 => "Right Arrow".to_string(),
            0x28 => "Down Arrow".to_string(),
            0x2D => "Insert".to_string(),
            0x2E => "Delete".to_string(),
            0x30 => "0".to_string(),
            0x31 => "1".to_string(),
            0x32 => "2".to_string(),
            0x33 => "3".to_string(),
            0x34 => "4".to_string(),
            0x35 => "5".to_string(),
            0x36 => "6".to_string(),
            0x37 => "7".to_string(),
            0x38 => "8".to_string(),
            0x39 => "9".to_string(),
            0x41 => "A".to_string(),
            0x42 => "B".to_string(),
            0x43 => "C".to_string(),
            0x44 => "D".to_string(),
            0x45 => "E".to_string(),
            0x46 => "F".to_string(),
            0x47 => "G".to_string(),
            0x48 => "H".to_string(),
            0x49 => "I".to_string(),
            0x4A => "J".to_string(),
            0x4B => "K".to_string(),
            0x4C => "L".to_string(),
            0x4D => "M".to_string(),
            0x4E => "N".to_string(),
            0x4F => "O".to_string(),
            0x50 => "P".to_string(),
            0x51 => "Q".to_string(),
            0x52 => "R".to_string(),
            0x53 => "S".to_string(),
            0x54 => "T".to_string(),
            0x55 => "U".to_string(),
            0x56 => "V".to_string(),
            0x57 => "W".to_string(),
            0x58 => "X".to_string(),
            0x59 => "Y".to_string(),
            0x5A => "Z".to_string(),
            0x60 => "Numpad 0".to_string(),
            0x61 => "Numpad 1".to_string(),
            0x62 => "Numpad 2".to_string(),
            0x63 => "Numpad 3".to_string(),
            0x64 => "Numpad 4".to_string(),
            0x65 => "Numpad 5".to_string(),
            0x66 => "Numpad 6".to_string(),
            0x67 => "Numpad 7".to_string(),
            0x68 => "Numpad 8".to_string(),
            0x69 => "Numpad 9".to_string(),
            0x6A => "Numpad *".to_string(),
            0x6B => "Numpad +".to_string(),
            0x6C => "Numpad ,".to_string(),
            0x6D => "Numpad -".to_string(),
            0x6E => "Numpad .".to_string(),
            0x6F => "Numpad /".to_string(),
            0x70 => "F1".to_string(),
            0x71 => "F2".to_string(),
            0x72 => "F3".to_string(),
            0x73 => "F4".to_string(),
            0x74 => "F5".to_string(),
            0x75 => "F6".to_string(),
            0x76 => "F7".to_string(),
            0x77 => "F8".to_string(),
            0x78 => "F9".to_string(),
            0x79 => "F10".to_string(),
            0x7A => "F11".to_string(),
            0x7B => "F12".to_string(),
            0x90 => "Num Lock".to_string(),
            0x91 => "Scroll Lock".to_string(),
            0xA0 => "Left Shift".to_string(),
            0xA1 => "Right Shift".to_string(),
            0xA2 => "Left Ctrl".to_string(),
            0xA3 => "Right Ctrl".to_string(),
            0xA4 => "Left Alt".to_string(),
            0xA5 => "Right Alt".to_string(),
            _ => format!("VK_{:#X}", vk),
        }
    }

    pub fn start_hotkey_listener(hotkey_vk: Arc<RwLock<u32>>, running: Arc<AtomicBool>) {
        std::thread::spawn(move || {
            loop {
                let vk = *hotkey_vk.read();
                if vk != 0 {
                    let state = unsafe { GetAsyncKeyState(vk as i32) };
                    if (state & 0x1) != 0 {
                        let new = !running.load(Ordering::SeqCst);
                        running.store(new, Ordering::SeqCst);
                        std::thread::sleep(std::time::Duration::from_millis(250));
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(30));
            }
        });
    }
}