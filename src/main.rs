mod app;
mod window_manager;
mod hotkey_manager;
mod clicker;
mod ui;

use app::MyApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        vsync: true,
        ..Default::default()
    };

    eframe::run_native(
        "🚀 Be_Clicker",
        options,
        Box::new(|_| Ok(Box::new(MyApp::new())))
    )
}