use crate::trocr_processor::ImageProcessor;
use candle_core::{DType, Device, Error, Tensor};
use candle_examples::token_output_stream::TokenOutputStream;
use candle_nn::VarBuilder;
use candle_transformers::models::{trocr, vit};
use image::{ImageBuffer, Rgb};
use tokenizers::Tokenizer;

pub struct TrOCRImplementationHandtux {
    tokenizer: Tokenizer,
    processor: ImageProcessor,
    device: Device,
    model: trocr::TrOCRModel,
    config: Config,
    tokenizer_dec: TokenOutputStream,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct Config {
    encoder: vit::Config,
    decoder: trocr::TrOCRConfig,
}

impl TrOCRImplementationHandtux {
    pub fn new() -> Self {
        let api = hf_hub::api::sync::Api::new().unwrap();
        let tokenizer_path = api
            .model(String::from("ToluClassics/candle-trocr-tokenizer"))
            .get("tokenizer.json")
            .unwrap();
        let tokenizer = Tokenizer::from_file(tokenizer_path.clone()).unwrap();
        let tokenizer_dec = TokenOutputStream::new(Tokenizer::from_file(tokenizer_path).unwrap());

        let device = Device::cuda_if_available(0).unwrap(); // What is an ordinal ???

        let (repo, branch) = ("microsoft/trocr-base-handwritten", "refs/pr/3");

        let model_path = api
            .repo(hf_hub::Repo::with_revision(
                repo.to_string(),
                hf_hub::RepoType::Model,
                branch.to_string(),
            ))
            .get("model.safetensors")
            .unwrap();

        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&[model_path], DType::F32, &device).unwrap()
        };

        let config_filename = api
            .repo(hf_hub::Repo::with_revision(
                repo.to_string(),
                hf_hub::RepoType::Model,
                branch.to_string(),
            ))
            .get("config.json")
            .unwrap();
        let config: Config =
            serde_json::from_reader(std::fs::File::open(config_filename.clone()).unwrap()).unwrap();
        let (encoder_config, decoder_config) = (&config.encoder, &config.decoder);
        let model = trocr::TrOCRModel::new(encoder_config, decoder_config, vb).unwrap();

        Self {
            tokenizer,
            processor: ImageProcessor::new(),
            device,
            model,
            config,
            tokenizer_dec,
        }
    }

    pub fn get_candidates(
        &mut self,
        image: ImageBuffer<Rgb<u8>, Vec<u8>>,
    ) -> Result<Vec<String>, Error> {
        println!("Preprocessing image...");
        let preprocessed = self
            .processor
            .preprocess(image)?
            .unsqueeze(0)?
            .to_device(&self.device)?;
        println!("Image preprocessed!");

        // Encode the image
        println!("Encoding image...");
        let encoder_xs = self.model.encoder().forward(&preprocessed)?;
        println!("Image encoded!");

        // Generate the text
        println!("Generating text...");
        let mut logits_processor =
            candle_transformers::generation::LogitsProcessor::new(1337, None, None);
        let mut token_ids = vec![self.config.decoder.decoder_start_token_id.clone()];
        for index in 0..128 {
            let context_size = if index >= 1 { 1 } else { token_ids.len() };
            let start_pos = token_ids.len().saturating_sub(context_size);
            let input_ids = Tensor::new(&token_ids[start_pos..], &self.device)?.unsqueeze(0)?;
            let logits = self.model.decode(&input_ids, &encoder_xs, start_pos)?;
            let logits = logits.squeeze(0)?;
            let logits = logits.get(logits.dim(0)? - 1)?;
            let token = logits_processor.sample(&logits)?;
            token_ids.push(token);

            if let Some(t) = self.tokenizer_dec.next_token(token)? {
                use std::io::Write;
                print!("{t}");
                std::io::stdout().flush()?;
            }
            if token == self.config.decoder.eos_token_id {
                break;
            }
        }

        Ok(vec!["Grah".to_string(), "Grah".to_string()])
    }
}
