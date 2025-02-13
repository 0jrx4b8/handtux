use std::sync::Arc;
use tokio::sync::Mutex;

use eframe::egui::{self, Pos2};
use egui::{Align, Layout, RichText, TopBottomPanel};
use tokio::sync::mpsc::Receiver;

use crate::trocr_model;

pub struct HandtuxUI {
    app_status: char, // L = Loading, R = Ready, T = thinking, S = Selecting
    painting_frame: Vec<[Pos2; 2]>,
    candidates: Vec<String>,
    trocr_model: Arc<Mutex<trocr_model::TrOCRImplementationHandtux>>,
    data_rx: Receiver<Vec<String>>,
    status_rx: Receiver<char>,
}

impl HandtuxUI {
    pub fn new(
        trocr_model: Arc<Mutex<trocr_model::TrOCRImplementationHandtux>>,
        data_rx: Receiver<Vec<String>>,
        status_rx: Receiver<char>,
    ) -> Self {
        Self {
            app_status: 'L',
            painting_frame: vec![],
            candidates: vec![],
            trocr_model,
            data_rx,
            status_rx,
        }
    }
}

impl eframe::App for HandtuxUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(&ctx, |_ui| {
            while let Ok(status) = self.status_rx.try_recv() {
                self.app_status = status;
            }

            if self.app_status == 'L' {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.label(RichText::new("Loading...").size(40.));
                });
            } else {
                while let Ok(candidates) = self.data_rx.try_recv() {
                    self.candidates = candidates;
                    self.app_status = 'S';
                }

                TopBottomPanel::top("top_panel").show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Candidates:");
                        for candidate in self.candidates.iter() {
                            if ui.button(candidate).clicked() {
                                println!("Suggestion clicked: {}", candidate);
                                self.painting_frame.clear();
                                self.app_status = 'R';
                            }
                        }
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui.button("Recognize").clicked() {
                                let model_arc = self.trocr_model.clone();
                                let m_painting_frame = self.painting_frame.clone();

                                tokio::spawn(async move {
                                    println!("Recognizing...");
                                    let _ = model_arc
                                        .lock()
                                        .await
                                        .get_candidates(&m_painting_frame)
                                        .await;
                                });
                            }
                            if ui.button("Options").clicked() {
                                println!("Options clicked");
                            }
                        });
                    });

                    if self.painting_frame.is_empty() {
                        ui.label(if self.app_status == 'T' {
                            "Thinking..."
                        } else {
                            "Write here"
                        });
                    }
                });

                egui::CentralPanel::default().show(ctx, |ui| {
                    let response =
                        ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::drag());
                    let painter = ui.painter();

                    if response.is_pointer_button_down_on() {
                        if let Some(pointer_pos) = response.interact_pointer_pos() {
                            //println!("Pointer pos: {:?}", pointer_pos);
                            self.painting_frame
                                .push([pointer_pos - response.drag_delta(), pointer_pos]);
                        }
                    }

                    for [start, end] in &self.painting_frame {
                        painter.line_segment(
                            [*start, *end],
                            (
                                1.0,
                                if self.app_status == 'T' || self.app_status == 'S' {
                                    egui::Color32::GOLD
                                } else {
                                    egui::Color32::WHITE
                                },
                            ),
                        );
                    }
                });
            }
        });

        ctx.request_repaint();
    }
}
