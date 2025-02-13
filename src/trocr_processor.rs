use image::{ImageBuffer, Rgb, RgbImage};
use eframe::egui::Pos2;

use candle_core::{Device, Result, Tensor};

pub struct ImageProcessor {
    height: usize,
    width: usize,
}

impl ImageProcessor {
    pub fn new() -> Self {
        Self {
            height: 384,
            width: 384,
        }
    }

    pub fn preprocess(&self, painting_frame: &Vec<[Pos2; 2]>) -> Result<Tensor> {
        print!("Converting painting frame to image... ");
        let image = self.painting_frame_to_image(painting_frame);
        println!("Image converted!");
        
        print!("Asserting dimensions... ");
        assert_eq!(image.dimensions(), (self.width as u32, self.height as u32));
        println!("Dimensions correct!");

        print!("Normalizing image... ");
        let normalized = self.normalize(image);
        match normalized {
            Ok(normalized) => {
                println!("Image normalized!");
                Ok(normalized)
            }
            Err(e) => {
                println!("Error normalizing image: {}", e);
                Err(e)
            }
        }
    }

    fn normalize(&self, m_image: ImageBuffer<Rgb<u8>, Vec<u8>>) -> Result<Tensor> {
        // Turning the image into a Tensor
        let mut pixels = Vec::new();
        for pixel in m_image.pixels() {
            pixels.push(pixel[0] as f32 / 255.0);
            pixels.push(pixel[1] as f32 / 255.0);
            pixels.push(pixel[2] as f32 / 255.0);
        }
        let image_tensor = Tensor::from_vec(
            pixels,
            (3, self.height, self.width),
            &Device::Cpu,
        )?;

        // Normalizing the image tensor
        let mean = Tensor::new(&[0.5f32, 0.5f32, 0.5f32], &Device::Cpu)?.reshape((3, 1, 1))?.expand((3, self.height, self.width))?;
        let std = Tensor::new(&[0.5f32, 0.5f32, 0.5f32], &Device::Cpu)?.reshape((3, 1, 1))?.expand((3, self.height, self.width))?;
        let normalized_tensor = (image_tensor - &mean)? / &std;

        normalized_tensor
        //Err(candle_core::Error::msg("Not implemented yet."))
    }
    
    // TODO: Rewrite the algorithm to not get out of bounds
    pub fn painting_frame_to_image(
        &self,
        painting_frame: &Vec<[Pos2; 2]>,
    ) -> image::ImageBuffer<Rgb<u8>, Vec<u8>> {
        let mut img = RgbImage::new(self.width as u32, self.height as u32);
    
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
                if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
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
}
