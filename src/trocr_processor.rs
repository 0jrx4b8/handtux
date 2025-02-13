// Code shamelessely copied from
// https://github.com/huggingface/candle/blob/main/candle-examples/examples/trocr/image_processor.rs
// Hopefully the license lets me do this

use image::{ImageBuffer, Rgb};

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

    pub fn preprocess(&self, image: ImageBuffer<Rgb<u8>, Vec<u8>>) -> Result<Tensor> {
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
}
