use crate::trocr_processor::ImageProcessor;
use candle_core::{DType, Device, Error};
use candle_nn::VarBuilder;
use candle_transformers::models::trocr::{self, TrOCRConfig};
use image::{ImageBuffer, Rgb};
use tokenizers::Tokenizer;

pub struct TrOCRImplementationHandtux {
    tokenizer: Tokenizer,
    processor: ImageProcessor,
    device: Device,
}

impl TrOCRImplementationHandtux {
    pub fn new() -> Self {
        let api = hf_hub::api::sync::Api::new().unwrap();
        let tokenizer_path = api
            .model(String::from("ToluClassics/candle-trocr-tokenizer"))
            .get("tokenizer.json")
            .unwrap();
        let tokenizer = Tokenizer::from_file(tokenizer_path).unwrap();

        let device = Device::cuda_if_available(0).unwrap(); // What is an ordinal ???

        let (repo, branch) = ("microsoft/trocr-base-handwritten", "refs/pr/3");
        
        // let model_path = api
        //     .repo(hf_hub::Repo::with_revision(
        //         repo.to_string(),
        //         hf_hub::RepoType::Model,
        //         branch.to_string(),
        //     ))
        //     .get("model.safetensors")
        //     .unwrap();
        // let vb = unsafe {
        //     candle_nn::VarBuilder::from_mmaped_safetensors(&[model_path], DType::F32, &device)
        //         .unwrap()
        // };

        let config_filename = api
            .repo(hf_hub::Repo::with_revision(
                repo.to_string(),
                hf_hub::RepoType::Model,
                branch.to_string(),
            ))
            .get("config.json").unwrap();
        let config: TrOCRConfig = serde_json::from_reader(std::fs::File::open(config_filename).unwrap()).unwrap();
        let (encoder_config, decoder_config) = (config.encoder, config.decoder);
        let mut model = trocr::TrOCRModel::new(&encoder_config, &decoder_config, vb).unwrap();

        Self {
            tokenizer,
            processor: ImageProcessor::new(),
            device,
        }
    }

    pub fn infer(&self, image: ImageBuffer<Rgb<u8>, Vec<u8>>) -> Result<Vec<String>, Error> {
        let preprocessed = self.processor.preprocess(image)?;

        Ok(vec![])
    }
}
