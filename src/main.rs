// src/main.rs
use std::{
    sync::{Arc, atomic::{AtomicBool, Ordering}},
    thread,
    time::Duration,
};
use parking_lot::RwLock;
use eframe::egui;

use windows::core::BOOL;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowTextW, IsWindowVisible, PostMessageW,
    WM_LBUTTONDOWN, WM_LBUTTONUP,
};
use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;

/// Callback для EnumWindows — заполняет Vec<(title, hwnd_as_isize)>
unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    if IsWindowVisible(hwnd).as_bool() {
        let mut buf = [0u16; 512];
        let len = unsafe { GetWindowTextW(hwnd, &mut buf) };
        if len > 0 {
            let title = String::from_utf16_lossy(&buf[..len as usize]);
            let vec = unsafe { &mut *(lparam.0 as *mut Vec<(String, isize)>) };
            vec.push((title, hwnd.0 as isize));
        }
    }
    BOOL(1)
}

/// Возвращает список видимых окон как (title, hwnd_as_isize)
fn get_windows_list() -> Vec<(String, isize)> {
    let mut list: Vec<(String, isize)> = Vec::new();
    unsafe {
        EnumWindows(Some(enum_proc), LPARAM(&mut list as *mut _ as isize)).ok();
    }
    list
}

/// Вспомогательная таблица: метка -> виртуальный key code (VK)
fn vk_for_label(label: &str) -> u32 {
    match label {
        "F6" => 0x75,   // VK_F6 = 0x75
        "F7" => 0x76,
        "F8" => 0x77,
        "F9" => 0x78,
        "F10" => 0x79,
        "R" => 0x52,
        "PgDn" => 0x22, // VK_NEXT
        _ => 0x75,
    }
}

fn label_for_vk(vk: u32) -> String {
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

struct MyApp {
    windows: Vec<(String, isize)>,
    selected_hwnd: Arc<RwLock<Option<isize>>>, // хранится как isize для потокобезопасности
    hotkey_vk: Arc<RwLock<u32>>,               // виртуальный код клавиши (VK)
    running: Arc<AtomicBool>,
    interval_ms: Arc<RwLock<u64>>,             // интервал между кликами в мс
}

impl MyApp {
    fn new() -> Self {
        let windows = get_windows_list();
        let selected_hwnd = Arc::new(RwLock::new(None));
        let hotkey_vk = Arc::new(RwLock::new(vk_for_label("F6"))); // по умолчанию F6
        let running = Arc::new(AtomicBool::new(false));
        let interval_ms = Arc::new(RwLock::new(500u64)); // 500 ms

        // Поток, который опрашивает GetAsyncKeyState для выбранной VK
        {
            let hotkey_vk = hotkey_vk.clone();
            let running = running.clone();
            thread::spawn(move || {
                loop {
                    let vk = *hotkey_vk.read();
                    if vk != 0 {
                        // GetAsyncKeyState возвращает SHORT; проверяем низкий бит (нажатие)
                        let state = unsafe { GetAsyncKeyState(vk as i32) };
                        if (state & 0x1) != 0 {
                            // toggle running
                            let new = !running.load(Ordering::SeqCst);
                            running.store(new, Ordering::SeqCst);
                            if new { println!("Автоклик: ВКЛ"); } else { println!("Автоклик: ВЫКЛ"); }
                            // небольшая задержка, чтобы не считывать одно нажатие много раз
                            thread::sleep(Duration::from_millis(250));
                        }
                    }
                    thread::sleep(Duration::from_millis(30));
                }
            });
        }

        // Поток-кликер: посылает сообщения в выбранное окно (если running == true)
        {
            let running = running.clone();
            let selected_hwnd = selected_hwnd.clone();
            let interval_ms = interval_ms.clone();
            thread::spawn(move || {
                loop {
                    if running.load(Ordering::SeqCst) {
                        if let Some(hwnd_val) = *selected_hwnd.read() {
                            let hwnd = HWND(hwnd_val as *mut _);
                            unsafe {
                                // левый клик (если нужен правый — заменить на WM_RBUTTON*)
                                PostMessageW(Some(hwnd), WM_LBUTTONDOWN, WPARAM(1), LPARAM(0)).ok();
                                PostMessageW(Some(hwnd), WM_LBUTTONUP, WPARAM(0), LPARAM(0)).ok();
                            }
                        }
                        let ms = *interval_ms.read();
                        thread::sleep(Duration::from_millis(ms));
                    } else {
                        thread::sleep(Duration::from_millis(60));
                    }
                }
            });
        }

        Self { windows, selected_hwnd, hotkey_vk, running, interval_ms }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Обновить список окон").clicked() {
                    self.windows = get_windows_list();
                }
                if ui.button(if self.running.load(Ordering::SeqCst) { "Стоп" } else { "Старт" }).clicked() {
                    let now = !self.running.load(Ordering::SeqCst);
                    self.running.store(now, Ordering::SeqCst);
                }
                ui.label(format!("Горячая: {}", label_for_vk(*self.hotkey_vk.read())));
                ui.label(format!("Интервал: {} ms", *self.interval_ms.read()));
                ui.label("By X_THEBEST_ wine lover puvin");
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Autoclicker");
            ui.separator();

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label("Окна (нажми чтобы выбрать):");
                    egui::ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
                        for (title, hwnd_val) in &self.windows {
                            let selected = *self.selected_hwnd.read() == Some(*hwnd_val);
                            if ui.selectable_label(selected, title).clicked() {
                                *self.selected_hwnd.write() = Some(*hwnd_val);
                            }
                        }
                    });
                });

                ui.vertical(|ui| {
                    ui.label("Клавиша Старт/Стоп:");
                    for label in &["F6","F7","F8","F9","F10", "R","PgDn"] {
                        let vk = vk_for_label(label);
                        let sel = *self.hotkey_vk.read() == vk;
                        if ui.selectable_label(sel, *label).clicked() {
                            *self.hotkey_vk.write() = vk;
                        }
                    }

                    ui.separator();
                    ui.label("Интервал между кликами (мс):");
                    let mut cur = *self.interval_ms.read() as i32;
                    if ui.add(egui::Slider::new(&mut cur, 50..=2000)).changed() {
                        *self.interval_ms.write() = cur as u64;
                    }

                    ui.separator();
                    ui.label(format!("Статус: {}", if self.running.load(Ordering::SeqCst) { "ВКЛ" } else { "ВЫКЛ" }));
                    ui.label("Клик идет в выбранное окно (оно не должно быть свернуто).");
                });
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native("Autoclicker", options, Box::new(|_| Ok(Box::new(MyApp::new()))))
}