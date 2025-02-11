use eframe::egui;

mod ui;
mod processing;

fn main() -> eframe::Result {
    pyo3::prepare_freethreaded_python();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([500.0, 200.0]),
        ..Default::default()
    };

    eframe::run_native("handtux", options, Box::new(|_cc| {
        Ok(Box::<ui::HandtuxUI>::default())
    }))
}
