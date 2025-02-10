use eframe::egui::{self, Pos2};
use egui::{Align, Layout, TopBottomPanel};
use image::{Rgb, RgbImage};
use pyo3::{prelude::*, types::PyBytes};
use std::io::Cursor;
use std::ffi::CString;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_painting_frame() {
        let painting_frame: Vec<[Pos2; 2]> = vec![];
        let img = painting_frame_to_image(&painting_frame, 100, 100);
        assert_eq!(img.dimensions(), (100, 100));
        // Check all pixels are white
        for pixel in img.pixels() {
            assert_eq!(*pixel, Rgb([255, 255, 255]));
        }
    }

    #[test]
    fn test_single_line() {
        let painting_frame = vec![[Pos2::new(10.0, 10.0), Pos2::new(20.0, 20.0)]];
        let img = painting_frame_to_image(&painting_frame, 50, 50);
        assert_eq!(img.dimensions(), (50, 50));
        // Check some pixels along the line are black
        assert_eq!(img.get_pixel(10, 10), &Rgb([0, 0, 0]));
        assert_eq!(img.get_pixel(15, 15), &Rgb([0, 0, 0]));
        assert_eq!(img.get_pixel(20, 20), &Rgb([0, 0, 0]));
    }

    #[test]
    fn test_out_of_bounds() {
        let painting_frame = vec![[Pos2::new(-10.0, -10.0), Pos2::new(200.0, 200.0)]];
        let img = painting_frame_to_image(&painting_frame, 100, 100);
        assert_eq!(img.dimensions(), (100, 100));
    }
}

lazy_static::lazy_static! {
    static ref OCR_MODULE: PyResult<Py<PyModule>> = Python::with_gil(|py| {
        let code = r#"
from transformers import TrOCRProcessor, VisionEncoderDecoderModel
from PIL import Image
import io
import numpy as np

# Initialize model once
processor = TrOCRProcessor.from_pretrained("microsoft/trocr-base-handwritten")
model = VisionEncoderDecoderModel.from_pretrained("microsoft/trocr-base-handwritten")

def process_image(image_bytes: bytes) -> list:
    # Convert bytes to image with exact size handling
    image = Image.open(io.BytesIO(image_bytes)).convert("RGB")
    
    # Process and run inference
    pixel_values = processor(image, return_tensors="pt").pixel_values
    generated_ids = model.generate(pixel_values)
    
    # Return multiple candidates (modify if you need beam search results)
    return [processor.batch_decode(generated_ids, skip_special_tokens=True)[0]]
"#;
        PyModule::from_code(
            py,
            CString::new(code).expect("Failed to convert code to CString").as_ref(),
            CString::new("ocr_module.py").expect("Failed to create module path").as_ref(),
            CString::new("ocr_module").expect("Failed to create module name").as_ref(),
        )
        .map(|m| m.into())
    });
}

fn painting_frame_to_image(
    painting_frame: &Vec<[Pos2; 2]>,
    width: u32,
    height: u32,
) -> image::ImageBuffer<Rgb<u8>, Vec<u8>> {
    let mut img = RgbImage::new(width, height);

    for pixel in img.pixels_mut() {
        *pixel = Rgb([255, 255, 255]);
    }

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
                img.put_pixel(x as u32, y as u32, Rgb([0, 0, 0]));
                img.put_pixel((x+1) as u32, y as u32, Rgb([0, 0, 0]));
                img.put_pixel(x as u32, (y+1) as u32, Rgb([0, 0, 0]));
                img.put_pixel((x+1) as u32, (y+1) as u32, Rgb([0, 0, 0]));
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
    // Convert ImageBuffer to PNG bytes
    let mut bytes: Vec<u8> = Vec::new();
    image
        .write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)
        .expect("Failed to encode image");

    Python::with_gil(|py| {
        // Get the preloaded OCR module
        let ocr_module = OCR_MODULE.as_ref().expect("OCR module failed to load");

        // Call Python function with image bytes
        let result: Vec<String> = ocr_module
            .call_method1(py, "process_image", (PyBytes::new(py, &bytes),))
            .expect("Python call failed")
            .extract(py)
            .expect("Failed to extract results");

        result
    })
}

fn main() -> eframe::Result {
    pyo3::prepare_freethreaded_python();


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
                            let img = painting_frame_to_image(&painting_frame, 500, 200);
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
