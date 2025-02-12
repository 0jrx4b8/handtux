use eframe::egui::Pos2;
use image::{Rgba, RgbaImage};
use kalosm_ocr::*;

pub fn painting_frame_to_image(
    painting_frame: &Vec<[Pos2; 2]>,
    width: u32,
    height: u32,
) -> image::ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut img = RgbaImage::new(width, height);

    for pixel in img.pixels_mut() {
        *pixel = Rgba([255, 255, 255, 255]);
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
                img.put_pixel(x as u32, y as u32, Rgba([0, 0, 0, 255]));
                img.put_pixel((x + 1) as u32, y as u32, Rgba([0, 0, 0, 255]));
                img.put_pixel(x as u32, (y + 1) as u32, Rgba([0, 0, 0, 255]));
                img.put_pixel((x + 1) as u32, (y + 1) as u32, Rgba([0, 0, 0, 255]));
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

pub async fn image_to_text(image: image::ImageBuffer<Rgba<u8>, Vec<u8>>) -> Vec<String> {
    println!("loading model...");
    let mut model = Ocr::builder().build().await.unwrap();
    println!("model loaded");
    println!("recognizing text...");
    let text = model.recognize_text(OcrInferenceSettings::new(image));
    println!("text recognized");
    
    println!("Text: {:?}", text);
    vec![text.unwrap()]
}
