use eframe::egui;
use egui::{TopBottomPanel, Layout, Align, Button, Separator};

fn main() -> eframe::Result {
    println!("Hello, world!");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([500.0, 200.0]),
        ..Default::default()
    };

    eframe::run_simple_native("app_name", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(&ctx, |ui| {
            TopBottomPanel::top("top_panel").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Suggestions:");
                    for suggestion in &["Hello", "World", "Rust", "Egui"] {
                        _ = ui.button(*suggestion);
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button("Options").clicked() {
                            println!("Options clicked");
                        }
                    });
                });
            });

            egui::CentralPanel::default().show(ctx, |ui| {
                ui.label("Draw here:");
                ui.separator();
                
            });
        });
    })
}
