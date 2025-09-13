mod app;
mod window_manager;
mod hotkey_manager;
mod clicker;

// src/main.rs
use std::{
    sync::{Arc, atomic::{AtomicBool, Ordering}},
    thread,
    time::Duration,
};
use parking_lot::RwLock;
use eframe::egui;
use egui::{Color32, RichText, Stroke};

use windows::core::BOOL;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowTextW, IsWindowVisible, PostMessageW,
    WM_LBUTTONDOWN, WM_LBUTTONUP,
};
use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;

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

fn vk_to_key_name(vk: u32) -> String {
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

struct MyApp {
    windows: Vec<(String, isize)>,
    selected_hwnd: Arc<RwLock<Option<isize>>>,
    hotkey_vk: Arc<RwLock<u32>>,
    running: Arc<AtomicBool>,
    interval_ms: Arc<RwLock<u64>>,
    animation_progress: f32,
    last_update: std::time::Instant,
    show_windows_list: bool,
    listening_for_key: bool,
    last_key_press: Option<u32>,
    windows_list_animation: f32,
}

impl MyApp {
    fn new() -> Self {
        let windows = get_windows_list();
        let selected_hwnd = Arc::new(RwLock::new(None));
        let hotkey_vk = Arc::new(RwLock::new(vk_for_label("F6")));
        let running = Arc::new(AtomicBool::new(false));
        let interval_ms = Arc::new(RwLock::new(500u64));
        let animation_progress = 0.0;
        let last_update = std::time::Instant::now();
        let show_windows_list = false;
        let listening_for_key = false;
        let last_key_press = None;
        let windows_list_animation = 0.0;

        // Hotkey polling thread
        {
            let hotkey_vk = hotkey_vk.clone();
            let running = running.clone();
            thread::spawn(move || {
                loop {
                    let vk = *hotkey_vk.read();
                    if vk != 0 {
                        let state = unsafe { GetAsyncKeyState(vk as i32) };
                        if (state & 0x1) != 0 {
                            let new = !running.load(Ordering::SeqCst);
                            running.store(new, Ordering::SeqCst);
                            thread::sleep(Duration::from_millis(250));
                        }
                    }
                    thread::sleep(Duration::from_millis(30));
                }
            });
        }

        // Clicker thread
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

        Self {
            windows,
            selected_hwnd,
            hotkey_vk,
            running,
            interval_ms,
            animation_progress,
            last_update,
            show_windows_list,
            listening_for_key,
            last_key_press,
            windows_list_animation,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update animation
        let now = std::time::Instant::now();
        let delta_time = now.duration_since(self.last_update).as_secs_f32();
        self.last_update = now;

        if self.running.load(Ordering::SeqCst) {
            self.animation_progress = (self.animation_progress + delta_time * 2.0) % 1.0;
        } else {
            self.animation_progress = 0.0;
        }

        // Анимация списка окон
        let target_animation = if self.show_windows_list { 1.0 } else { 0.0 };
        let animation_speed = 8.0 * delta_time;
        self.windows_list_animation += (target_animation - self.windows_list_animation) * animation_speed;

        // Key listening logic
        if self.listening_for_key {
            for vk in 1..255 {
                let state = unsafe { GetAsyncKeyState(vk as i32) };
                if (state & 0x1) != 0 && vk != 0x01 { // Ignore mouse left click
                    *self.hotkey_vk.write() = vk as u32;
                    self.listening_for_key = false;
                    self.last_key_press = Some(vk as u32);
                    break;
                }
            }
        }

        // Custom styling
        let mut style = (*ctx.style()).clone();
        style.visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(25, 25, 35);
        style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(45, 45, 60);
        style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(55, 55, 75);
        style.visuals.widgets.active.bg_fill = Color32::from_rgb(65, 65, 90);
        style.visuals.widgets.open.bg_fill = Color32::from_rgb(75, 75, 100);
        style.visuals.selection.bg_fill = Color32::from_rgb(80, 120, 200);
        style.visuals.window_fill = Color32::from_rgb(30, 30, 40);
        style.visuals.window_stroke = Stroke::new(1.0, Color32::from_rgb(60, 60, 80));
        style.visuals.panel_fill = Color32::from_rgb(35, 35, 45);
        style.visuals.faint_bg_color = Color32::from_rgb(40, 40, 55);
        style.visuals.extreme_bg_color = Color32::from_rgb(20, 20, 30);
        style.visuals.code_bg_color = Color32::from_rgb(40, 40, 55);
        style.visuals.window_corner_radius = 8.0.into();
        style.visuals.widgets.noninteractive.corner_radius = 6.0.into();
        ctx.set_style(style);

        // Top panel
        egui::TopBottomPanel::top("top_panel")
            .exact_height(40.0)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.heading(RichText::new("🚀 Autoclicker").color(Color32::from_rgb(120, 180, 255)));
                    ui.separator();

                    let is_running = self.running.load(Ordering::SeqCst);
                    let status_color = if is_running {
                        Color32::from_rgb(0, 200, 100)
                    } else {
                        Color32::from_rgb(200, 80, 80)
                    };

                    let pos = ui.cursor().min;
                    ui.painter().circle_filled(
                        pos + egui::vec2(8.0, 8.0),
                        6.0,
                        if is_running && (self.animation_progress > 0.5) {
                            Color32::from_rgb(0, 150, 80)
                        } else {
                            status_color
                        }
                    );
                    ui.add_space(16.0);

                    ui.label(RichText::new(if is_running { "ВКЛ" } else { "ВЫКЛ" })
                        .color(status_color)
                        .strong());

                    ui.separator();
                    ui.label(RichText::new(format!("Горячая клавиша: {}", vk_to_key_name(*self.hotkey_vk.read())))
                        .color(Color32::from_rgb(180, 180, 200)));

                    ui.separator();
                    ui.label(RichText::new(format!("Интервал: {} ms", *self.interval_ms.read()))
                        .color(Color32::from_rgb(180, 180, 200)));
                });
            });

        // Main content
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);

                // Control buttons
                ui.horizontal(|ui| {
                    if ui.add(egui::Button::new(RichText::new("🔄 Обновить список").color(Color32::WHITE))
                        .fill(Color32::from_rgb(70, 100, 180))
                        .min_size(egui::vec2(150.0, 35.0)))
                        .clicked()
                    {
                        self.windows = get_windows_list();
                    }

                    let is_running = self.running.load(Ordering::SeqCst);
                    let (button_text, button_color) = if is_running {
                        ("⏹️ Стоп", Color32::from_rgb(200, 80, 80))
                    } else {
                        ("▶️ Старт", Color32::from_rgb(0, 180, 100))
                    };

                    if ui.add(egui::Button::new(RichText::new(button_text).color(Color32::WHITE))
                        .fill(button_color)
                        .min_size(egui::vec2(100.0, 35.0)))
                        .clicked()
                    {
                        self.running.store(!is_running, Ordering::SeqCst);
                    }
                });

                ui.add_space(15.0);
            });

            ui.separator();
            ui.add_space(10.0);

            // Main content layout
            ui.columns(2, |columns| {
                // Left column - Window list dropdown
                columns[0].group(|ui| {
                    // Dropdown header
                    let response = ui.add(egui::Button::new(
                        RichText::new(if self.show_windows_list { "📋 Список Окон ▼" } else { "📋 Список Окон ▶" })
                            .color(Color32::from_rgb(120, 180, 255))
                            .strong()
                    ).fill(Color32::from_rgb(50, 50, 70)).frame(false));

                    if response.clicked() {
                        self.show_windows_list = !self.show_windows_list;
                    }

                    // Animated dropdown content with smooth animation
                    if self.windows_list_animation > 0.01 {
                        ui.separator();

                        // Плавное появление через прозрачность и высоту
                        let alpha = self.windows_list_animation;
                        let height_factor = self.windows_list_animation;

                        ui.scope(|ui| {
                            // Анимированная высота
                            let max_height = 250.0 * height_factor;

                            egui::ScrollArea::vertical()
                                .max_height(max_height)
                                .show(ui, |ui| {
                                    for (title, hwnd_val) in &self.windows {
                                        let selected = *self.selected_hwnd.read() == Some(*hwnd_val);

                                        let response = ui.selectable_label(
                                            selected,
                                            RichText::new(title).color(if selected {
                                                Color32::WHITE
                                            } else {
                                                Color32::from_rgb(200, 200, 200)
                                            })
                                        );

                                        if response.clicked() {
                                            *self.selected_hwnd.write() = Some(*hwnd_val);
                                        }

                                        if response.hovered() {
                                            ui.painter().rect_filled(
                                                response.rect.expand(2.0),
                                                4.0,
                                                Color32::from_rgba_premultiplied(80, 120, 200, (30.0 * alpha) as u8)
                                            );
                                        }
                                    }
                                });
                        });
                    }

                    // Show selected window (тоже с анимацией)
                    if let Some(hwnd) = *self.selected_hwnd.read() {
                        if let Some((title, _)) = self.windows.iter().find(|(_, h)| *h == hwnd) {
                            ui.add_space(5.0 * self.windows_list_animation);
                            ui.separator();
                            ui.add_space(5.0 * self.windows_list_animation);
                            ui.label(RichText::new("Выбрано:").strong());
                            ui.label(RichText::new(title).color(Color32::from_rgb(120, 200, 120)));
                        }
                    }
                });

                // Right column - Settings
                columns[1].group(|ui| {
                    ui.heading(RichText::new("⚙️ Настройки").color(Color32::from_rgb(120, 180, 255)));
                    ui.separator();

                    ui.vertical(|ui| {
                        ui.label(RichText::new("Горячая клавиша:").strong());

                        // Custom key binding
                        ui.horizontal(|ui| {
                            let current_key = vk_to_key_name(*self.hotkey_vk.read());
                            let button_text = if self.listening_for_key {
                                "Нажмите любую клавишу..."
                            } else {
                                &current_key
                            };

                            let button_color = if self.listening_for_key {
                                Color32::from_rgb(200, 150, 50)
                            } else {
                                Color32::from_rgb(70, 100, 180)
                            };

                            if ui.add(egui::Button::new(RichText::new(button_text).color(Color32::WHITE))
                                .fill(button_color)
                                .min_size(egui::vec2(150.0, 35.0)))
                                .clicked()
                            {
                                self.listening_for_key = true;
                            }

                            let reset_button = ui.button("❌");
                            if reset_button.clicked() {
                                *self.hotkey_vk.write() = vk_for_label("F6");
                                self.listening_for_key = false;
                            }
                            reset_button.on_hover_text("Сбросить");
                        });

                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(10.0);

                        ui.label(RichText::new("Интервал между кликами (мс):").strong());
                        let mut cur = *self.interval_ms.read() as i32;
                        if ui.add(egui::Slider::new(&mut cur, 50..=2000)
                            .text_color(Color32::WHITE))
                            .changed()
                        {
                            *self.interval_ms.write() = cur as u64;
                        }

                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(10.0);

                        // Status info with animation
                        let is_running = self.running.load(Ordering::SeqCst);
                        ui.label(RichText::new("Статус:").strong());
                        ui.horizontal(|ui| {
                            let pulse = (self.animation_progress * 2.0 * std::f32::consts::PI).sin().abs();
                            let pulse_color = if is_running {
                                Color32::from_rgb(
                                    (0 + (55.0 * pulse) as u8).min(255),
                                    (200 + (55.0 * pulse) as u8).min(255),
                                    (100 + (55.0 * pulse) as u8).min(255),
                                )
                            } else {
                                Color32::from_rgb(200, 80, 80)
                            };

                            let pos = ui.cursor().min;
                            ui.painter().circle_filled(
                                pos + egui::vec2(8.0, 8.0),
                                8.0,
                                pulse_color
                            );
                            ui.add_space(16.0);

                            ui.label(RichText::new(if is_running { "АКТИВНО" } else { "ПАУЗА" })
                                .color(pulse_color)
                                .strong());
                        });

                        ui.add_space(5.0);
                        ui.label(RichText::new("Клик идет в выбранное окно (оно не должно быть свернуто)")
                            .color(Color32::from_rgb(150, 150, 170))
                            .small());
                    });
                });
            });

            // Footer
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.separator();
                ui.add_space(10.0);
                ui.label(RichText::new("By X_THEBEST_ wine lover puvin")
                    .color(Color32::from_rgb(150, 150, 170))
                    .small());
            });
        });

        // Request repaint for animations
        ctx.request_repaint();
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        vsync: true,
        ..Default::default()
    };

    eframe::run_native(
        "🚀 Autoclicker Pro",
        options,
        Box::new(|_| Ok(Box::new(MyApp::new())))
    )
}