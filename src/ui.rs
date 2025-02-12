use eframe::egui::{self, Pos2};
use egui::{Align, Layout, TopBottomPanel};

use crate::processing::{image_to_text, painting_frame_to_image};

pub struct HandtuxUI {
    painting_frame: Vec<[Pos2; 2]>,
    candidates: Vec<String>,
}

impl Default for HandtuxUI {
    fn default() -> Self {
        Self {
            painting_frame: vec![],
            candidates: vec![],
        }
    }
}

impl eframe::App for HandtuxUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(&ctx, |_ui| {
            TopBottomPanel::top("top_panel").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Candidates:");
                    for candidate in self.candidates.iter() {
                        if ui.button(candidate).clicked() {
                            println!("Suggestion clicked: {}", candidate);
                        }
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button("Recognize").clicked() {
                            let img = painting_frame_to_image(&self.painting_frame, 500, 200);
                            self.candidates = vec![];
                            tokio::spawn(async move {
                                let text = image_to_text(img).await;
                                println!("Text: {:?}", text);
                            });
                            self.painting_frame.clear();
                        }
                        if ui.button("Options").clicked() {
                            println!("Options clicked");
                        }
                    });
                });

                if self.painting_frame.is_empty() {
                    ui.label("Write here:");
                }
            });

            egui::CentralPanel::default().show(ctx, |ui| {
                let response =
                    ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::drag());
                let painter = ui.painter();

                if response.is_pointer_button_down_on() {
                    if let Some(pointer_pos) = response.interact_pointer_pos() {
                        //println!("Pointer pos: {:?}", pointer_pos);
                        self.painting_frame.push([pointer_pos - response.drag_delta(), pointer_pos]);
                    }
                }

                for [start, end] in &self.painting_frame {
                    painter.line_segment([*start, *end], (1.0, egui::Color32::WHITE));
                }
            });
        });
    }
}
