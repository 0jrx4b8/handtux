use std::sync::Arc;
use tokio::sync::Mutex;

use eframe::egui;

mod ui;

mod trocr_model;
mod trocr_processor;

#[tokio::main]
async fn main() -> eframe::Result {
    println!("Welcome!");
    let (data_tx, data_rx) = tokio::sync::mpsc::channel::<Vec<String>>(3);
    let (status_tx, status_rx) = tokio::sync::mpsc::channel::<char>(3);

    let trocr_model = Arc::new(Mutex::new(trocr_model::TrOCRImplementationHandtux::new()));

    let trocr_model_arc = trocr_model.clone();

    tokio::spawn(async move {
        trocr_model_arc.lock().await.init(data_tx, status_tx).await;
    });

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([500.0, 200.0]),
        ..Default::default()
    };

    eframe::run_native(
        "handtux",
        options,
        Box::new(|_cc| {
            Ok(Box::<ui::HandtuxUI>::new(ui::HandtuxUI::new(
                trocr_model,
                data_rx,
                status_rx,
            )))
        }),
    )
}
