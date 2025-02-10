use eframe::egui::{self, Pos2};
use egui::{Align, Layout, TopBottomPanel};
use image::{Rgb, RgbImage};

fn painting_frame_to_image(
    painting_frame: &Vec<[Pos2; 2]>,
    width: u32,
    height: u32,
) -> image::ImageBuffer<Rgb<u8>, Vec<u8>> {
    let mut img  = RgbImage::new(width, height);

    for [start, end] in painting_frame {
        let (x0, y0) = (start.x as i32, start.y as i32);
        let (x1, y1) = (end.x as i32, end.y as i32);

        // Bresenham's line algorithm to draw the line
        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        let mut x = x0;
        let mut y = y0;

        loop {
            if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
                img.put_pixel(x as u32, y as u32, Rgb([255, 255, 255]));
            }
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    img
}

fn image_to_text(image: image::ImageBuffer<Rgb<u8>, Vec<u8>>) -> Vec<String> {
    vec!["Hello".to_string(), "World".to_string()]
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([500.0, 200.0]),
        ..Default::default()
    };

    let mut painting_frame: Vec<[Pos2; 2]> = vec![];
    let mut suggestions: Vec<String> = vec![];

    eframe::run_simple_native("handtux", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(&ctx, |_ui| {
            TopBottomPanel::top("top_panel").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Suggestions:");
                    for suggestion in suggestions.iter() {
                        if ui.button(suggestion).clicked() {
                            println!("Suggestion clicked: {}", suggestion);
                        }
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button("Recognize").clicked() {
                            let img = painting_frame_to_image(
                                &painting_frame,
                                500,
                                200,
                            );
                            suggestions = image_to_text(img);
                            painting_frame.clear();
                        }
                        if ui.button("Options").clicked() {
                            println!("Options clicked");
                        }
                    });
                });

                if painting_frame.is_empty() {
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
                        painting_frame.push([pointer_pos - response.drag_delta(), pointer_pos]);
                    }
                }

                for [start, end] in &painting_frame {
                    painter.line_segment([*start, *end], (1.0, egui::Color32::WHITE));
                }
            });
        });
    })
}
