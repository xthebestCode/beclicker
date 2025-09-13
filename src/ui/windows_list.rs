use eframe::egui;
use egui::Color32;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use parking_lot::RwLock;

pub struct WindowsList {
    pub show_windows_list: bool,
    pub windows_list_animation: f32,
}

impl WindowsList {
    pub fn new() -> Self {
        Self {
            show_windows_list: false,
            windows_list_animation: 0.0,
        }
    }

    pub fn update_animation(&mut self, delta_time: f32, show_windows_list: bool) {
        let target_animation = if show_windows_list { 1.0 } else { 0.0 };
        let animation_speed = 8.0 * delta_time;
        self.windows_list_animation += (target_animation - self.windows_list_animation) * animation_speed;
    }

    pub fn render(
        &mut self,
        ui: &mut egui::Ui,
        windows: &[(String, isize)],
        selected_hwnd: &Arc<RwLock<Option<isize>>>,
    ) {
        // Dropdown header
        let response = ui.add(egui::Button::new(
            egui::RichText::new(if self.show_windows_list { "📋Список Окон:" } else { "📋Список Окон ▶" })
                .color(Color32::from_rgb(120, 180, 255))
                .strong()
        ).fill(Color32::from_rgb(50, 50, 70)).frame(false));

        if response.clicked() {
            self.show_windows_list = !self.show_windows_list;
        }

        // Animated dropdown content
        if self.windows_list_animation > 0.01 {
            ui.separator();

            let alpha = self.windows_list_animation;
            let height_factor = self.windows_list_animation;

            ui.scope(|ui| {
                let max_height = 250.0 * height_factor;

                egui::ScrollArea::vertical()
                    .max_height(max_height)
                    .show(ui, |ui| {
                        for (title, hwnd_val) in windows {
                            let selected = *selected_hwnd.read() == Some(*hwnd_val);

                            let response = ui.selectable_label(
                                selected,
                                egui::RichText::new(title).color(if selected {
                                    Color32::WHITE
                                } else {
                                    Color32::from_rgb(200, 200, 200)
                                })
                            );

                            if response.clicked() {
                                *selected_hwnd.write() = Some(*hwnd_val);
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

        // Show selected window
        if let Some(hwnd) = *selected_hwnd.read() {
            if let Some((title, _)) = windows.iter().find(|(_, h)| *h == hwnd) {
                ui.add_space(5.0 * self.windows_list_animation);
                ui.separator();
                ui.add_space(5.0 * self.windows_list_animation);
                ui.label(egui::RichText::new("Выбрано:").strong());
                ui.label(egui::RichText::new(title).color(Color32::from_rgb(120, 200, 120)));
            }
        }
    }
}