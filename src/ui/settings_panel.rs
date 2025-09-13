use eframe::egui;
use egui::{Color32, RichText};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use parking_lot::RwLock;

use crate::hotkey_manager::HotkeyManager;

pub struct SettingsPanel;

impl SettingsPanel {
    pub fn render(
        &self,
        ui: &mut egui::Ui,
        hotkey_vk: &Arc<RwLock<u32>>,
        interval_ms: &Arc<RwLock<u64>>,
        running: &Arc<AtomicBool>,
        animation_progress: f32,
        listening_for_key: &mut bool,
    ) {
        ui.heading(RichText::new("⚙ Настройки").color(Color32::from_rgb(120, 180, 255)));
        ui.separator();

        ui.vertical(|ui| {
            ui.label(RichText::new("Горячая клавиша:").strong());

            // Custom key binding
            ui.horizontal(|ui| {
                let current_key = HotkeyManager::vk_to_key_name(*hotkey_vk.read());
                let button_text = if *listening_for_key {
                    "Нажмите любую клавишу..."
                } else {
                    &current_key
                };

                let button_color = if *listening_for_key {
                    Color32::from_rgb(200, 150, 50)
                } else {
                    Color32::from_rgb(70, 100, 180)
                };

                if ui.add(egui::Button::new(RichText::new(button_text).color(Color32::WHITE))
                    .fill(button_color)
                    .min_size(egui::vec2(150.0, 35.0)))
                    .clicked()
                {
                    *listening_for_key = true;
                }

                let reset_button = ui.button("❌");
                if reset_button.clicked() {
                    *hotkey_vk.write() = HotkeyManager::vk_for_label("F6");
                    *listening_for_key = false;
                }
                reset_button.on_hover_text("Сбросить");
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            ui.label(RichText::new("Интервал между кликами (мс):").strong());
            let mut cur = *interval_ms.read() as i32;
            if ui.add(egui::Slider::new(&mut cur, 50..=2000)
                .text_color(Color32::WHITE))
                .changed()
            {
                *interval_ms.write() = cur as u64;
            }

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            // Status info with animation
            let is_running = running.load(Ordering::SeqCst);
            ui.label(RichText::new("Статус:").strong());
            ui.horizontal(|ui| {
                let pulse = (animation_progress * 2.0 * std::f32::consts::PI).sin().abs();
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

                ui.label(RichText::new(if is_running { " АКТИВНО" } else { " ПАУЗА" })
                    .color(pulse_color)
                    .strong());
            });

            ui.add_space(5.0);
            ui.label(RichText::new("Клик идет в выбранное окно.")
                .color(Color32::from_rgb(150, 150, 170))
                .small());
        });
    }
}