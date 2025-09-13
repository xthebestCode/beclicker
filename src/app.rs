use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use parking_lot::RwLock;
use eframe::egui;
use egui::{Color32, RichText, Stroke};

use crate::{
    window_manager::WindowManager,
    hotkey_manager::HotkeyManager,
    clicker::Clicker,
    ui::{top_panel::TopPanel, windows_list::WindowsList, settings_panel::SettingsPanel}
};

pub struct MyApp {
    windows: Vec<(String, isize)>,
    selected_hwnd: Arc<RwLock<Option<isize>>>,
    hotkey_vk: Arc<RwLock<u32>>,
    running: Arc<AtomicBool>,
    interval_ms: Arc<RwLock<u64>>,
    hold_shift: Arc<AtomicBool>,
    hold_ctrl: Arc<AtomicBool>,
    animation_progress: f32,
    last_update: std::time::Instant,
    listening_for_key: bool,
    last_key_press: Option<u32>,

    // UI components
    top_panel: TopPanel,
    windows_list: WindowsList,
    settings_panel: SettingsPanel,
}

impl MyApp {
    pub fn new() -> Self {
        let windows = WindowManager::get_windows_list();
        let selected_hwnd = Arc::new(RwLock::new(None));
        let hotkey_vk = Arc::new(RwLock::new(HotkeyManager::vk_for_label("F6")));
        let running = Arc::new(AtomicBool::new(false));
        let interval_ms = Arc::new(RwLock::new(500u64));
        let hold_shift = Arc::new(AtomicBool::new(false));
        let hold_ctrl = Arc::new(AtomicBool::new(false));
        let animation_progress = 0.0;
        let last_update = std::time::Instant::now();
        let listening_for_key = false;
        let last_key_press = None;

        HotkeyManager::start_hotkey_listener(hotkey_vk.clone(), running.clone());
        Clicker::start_clicker(
            running.clone(),
            selected_hwnd.clone(),
            interval_ms.clone(),
            hold_shift.clone(),
            hold_ctrl.clone(),
        );

        Self {
            windows,
            selected_hwnd,
            hotkey_vk,
            running,
            interval_ms,
            hold_shift,
            hold_ctrl,
            animation_progress,
            last_update,
            listening_for_key,
            last_key_press,

            top_panel: TopPanel,
            windows_list: WindowsList::new(),
            settings_panel: SettingsPanel,
        }
    }

    fn update_animations(&mut self, delta_time: f32) {
        if self.running.load(Ordering::SeqCst) {
            self.animation_progress = (self.animation_progress + delta_time * 2.0) % 1.0;
        } else {
            self.animation_progress = 0.0;
        }

        self.windows_list.update_animation(delta_time, self.windows_list.show_windows_list);
    }

    fn handle_key_listening(&mut self) {
        if self.listening_for_key {
            for vk in 1..255 {
                let state = unsafe { windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState(vk as i32) };
                if (state & 0x1) != 0 && vk != 0x01 {
                    *self.hotkey_vk.write() = vk as u32;
                    self.listening_for_key = false;
                    self.last_key_press = Some(vk as u32);
                    break;
                }
            }
        }
    }

    fn setup_style(&self, ctx: &egui::Context) {
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
    }

    fn render_control_buttons(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.add(egui::Button::new(egui::RichText::new("🔄 Обновить список").color(Color32::WHITE))
                .fill(Color32::from_rgb(70, 100, 180))
                .min_size(egui::vec2(150.0, 35.0)))
                .clicked()
            {
                self.windows = WindowManager::get_windows_list();
            }

            let is_running = self.running.load(Ordering::SeqCst);
            let (button_text, button_color) = if is_running {
                ("⏹ Стоп", Color32::from_rgb(200, 80, 80))
            } else {
                ("▶ Старт", Color32::from_rgb(0, 180, 100))
            };

            if ui.add(egui::Button::new(egui::RichText::new(button_text).color(Color32::WHITE))
                .fill(button_color)
                .min_size(egui::vec2(100.0, 35.0)))
                .clicked()
            {
                self.running.store(!is_running, Ordering::SeqCst);
            }
        });
    }

    fn render_modifier_buttons(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Модификаторы:").strong());

        let is_shift_held = self.hold_shift.load(Ordering::SeqCst);
        let (shift_text, shift_color) = if is_shift_held {
            ("🔒 LShift зажат", Color32::from_rgb(0, 180, 100))
        } else {
            ("🔓 LShift", Color32::from_rgb(100, 100, 100))
        };

        if ui.add(egui::Button::new(RichText::new(shift_text).color(Color32::WHITE))
            .fill(shift_color)
            .min_size(egui::vec2(120.0, 35.0)))
            .clicked()
        {
            self.hold_shift.store(!is_shift_held, Ordering::SeqCst);
        }

        ui.add_space(5.0);

        // Кнопка зажатия LCtrl
        let is_ctrl_held = self.hold_ctrl.load(Ordering::SeqCst);
        let (ctrl_text, ctrl_color) = if is_ctrl_held {
            ("🔒 LCtrl зажат", Color32::from_rgb(0, 180, 100))
        } else {
            ("🔓 LCtrl", Color32::from_rgb(100, 100, 100))
        };

        if ui.add(egui::Button::new(RichText::new(ctrl_text).color(Color32::WHITE))
            .fill(ctrl_color)
            .min_size(egui::vec2(120.0, 35.0)))
            .clicked()
        {
            self.hold_ctrl.store(!is_ctrl_held, Ordering::SeqCst);
        }

        ui.add_space(5.0);
        ui.label(RichText::new("Модификаторы остаются зажатыми постоянно")
            .color(Color32::from_rgb(150, 150, 170))
            .small());
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now = std::time::Instant::now();
        let delta_time = now.duration_since(self.last_update).as_secs_f32();
        self.last_update = now;

        self.update_animations(delta_time);
        self.handle_key_listening();
        self.setup_style(ctx);

        // Top panel
        egui::TopBottomPanel::top("top_panel")
            .exact_height(40.0)
            .show(ctx, |ui| {
                self.top_panel.render(
                    ui,
                    &self.running,
                    &self.hotkey_vk,
                    &self.interval_ms,
                    self.animation_progress
                );
            });

        // Main content
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                self.render_control_buttons(ui);
                ui.add_space(15.0);
            });

            ui.separator();
            ui.add_space(10.0);

            // Main content layout
            ui.columns(2, |columns| {
                // Left column - Window list dropdown
                columns[0].group(|ui| {
                    self.windows_list.render(
                        ui,
                        &self.windows,
                        &self.selected_hwnd
                    );
                });

                // Right column - Settings
                columns[1].group(|ui| {
                    self.settings_panel.render(
                        ui,
                        &self.hotkey_vk,
                        &self.interval_ms,
                        &self.running,
                        self.animation_progress,
                        &mut self.listening_for_key
                    );

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    // Кнопки модификаторов
                    self.render_modifier_buttons(ui);
                });
            });

            // Footer
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.separator();
                ui.add_space(10.0);
                ui.label(RichText::new("By @faworitewine wine lover puvin")
                    .color(Color32::from_rgb(150, 150, 170)));
                ui.label(RichText::new("Version 0.4")
                    .color(Color32::from_rgb(150, 150, 170)));
            });
        });

        ctx.request_repaint();
    }
}