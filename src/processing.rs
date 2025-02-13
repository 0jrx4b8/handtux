use eframe::egui::Pos2;
use image::{Rgb, RgbImage};

// lazy_static::lazy_static! {
//     static ref OCR_MODULE: PyResult<Py<PyModule>> = Python::with_gil(|py| {
//         let path = Path::new("python/ocr_module.py");
//         let code = std::fs::read_to_string(path).expect("Couldn't read the file...");
//         PyModule::from_code(
//             py,
//             CString::new(code).expect("Failed to convert code to CString").as_ref(),
//             CString::new("ocr_module.py").expect("Failed to create module path").as_ref(),
//             CString::new("ocr_module").expect("Failed to create module name").as_ref(),
//         )
//         .map(|m| m.into())
//     });
// }

pub fn painting_frame_to_image(
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
                img.put_pixel((x + 1) as u32, y as u32, Rgb([0, 0, 0]));
                img.put_pixel(x as u32, (y + 1) as u32, Rgb([0, 0, 0]));
                img.put_pixel((x + 1) as u32, (y + 1) as u32, Rgb([0, 0, 0]));
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

// pub fn image_to_text(image: image::ImageBuffer<Rgb<u8>, Vec<u8>>) -> Vec<String> {
//     // Convert ImageBuffer to PNG bytes
//     let mut bytes: Vec<u8> = Vec::new();
//     image
//         .write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)
//         .expect("Failed to encode image");

//     Python::with_gil(|py| {
//         // Get the preloaded OCR module
//         let ocr_module = OCR_MODULE.as_ref().expect("OCR module failed to load");

//         let engine = ocr_module
//             .getattr(py, "get_engine")
//             .expect("Missing get_engine")
//             .call0(py)
//             .expect("Failed to get engine");

//         engine
//             .call_method1(py, "process_image", (PyBytes::new(py, &bytes),))
//             .expect("Python call failed")
//             .extract(py)
//             .expect("Failed to extract results")
//     })
// }
