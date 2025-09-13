use eframe::egui;
use egui::{Color32, RichText};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use parking_lot::RwLock;

use crate::hotkey_manager::HotkeyManager;

pub struct TopPanel;

impl TopPanel {
    pub fn render(
        &self,
        ui: &mut egui::Ui,
        running: &Arc<AtomicBool>,
        hotkey_vk: &Arc<RwLock<u32>>,
        interval_ms: &Arc<RwLock<u64>>,
        animation_progress: f32,
    ) {
        ui.horizontal_centered(|ui| {
            ui.heading(RichText::new("🚀 Be_Cliker").color(Color32::from_rgb(120, 180, 255)));
            ui.separator();

            let is_running = running.load(Ordering::SeqCst);
            let status_color = if is_running {
                Color32::from_rgb(0, 200, 100)
            } else {
                Color32::from_rgb(200, 80, 80)
            };

            let pos = ui.cursor().min;
            ui.painter().circle_filled(
                pos + egui::vec2(8.0, 16.0),
                6.0,
                if is_running && (animation_progress > 0.5) {
                    Color32::from_rgb(0, 150, 80)
                } else {
                    status_color
                }
            );
            ui.add_space(16.0);

            ui.label(RichText::new(if is_running { " ВКЛ" } else { " ВЫКЛ" })
                .color(status_color)
                .strong());

            ui.separator();
            ui.label(RichText::new(format!("Горячая клавиша: {}", HotkeyManager::vk_to_key_name(*hotkey_vk.read())))
                .color(Color32::from_rgb(180, 180, 200)));

            ui.separator();
            ui.label(RichText::new(format!("Интервал: {} ms", *interval_ms.read()))
                .color(Color32::from_rgb(180, 180, 200)));
        });
    }
}