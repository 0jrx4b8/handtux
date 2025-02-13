use eframe::egui;

mod ui;

mod trocr_model;
mod trocr_processor;

fn main() -> eframe::Result {
    println!("Welcome! Loading TrOCR model...");
    let trocr_model = trocr_model::TrOCRImplementationHandtux::new();
    println!("TrOCR model loaded!");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([500.0, 200.0]),
        ..Default::default()
    };

    eframe::run_native(
        "handtux",
        options,
        Box::new(|_cc| Ok(Box::<ui::HandtuxUI>::new(ui::HandtuxUI::new(trocr_model)))),
    )
}
