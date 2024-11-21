// Code definitely copied from
// https://github.com/huggingface/candle/tree/main/candle-examples/examples/trocr

mod image_processor;

use candle_examples::token_output_stream::TokenOutputStream;
use candle_nn::VarBuilder;
use candle_core::{DType, Tensor};
use candle_transformers::models::{trocr, vit};
use candle_transformers::models::trocr::TrOCRModel;
use tokenizers::tokenizer::Tokenizer;

pub struct TrOcrCandleWrapper {
    model: TrOCRModel
}

impl TrOcrCandleWrapper {
    pub fn new(&self) -> Self {
        let api = hf_hub::api::sync::Api::new().unwrap();

        let tokenizer_file = api
            .model(String::from("ToluClassics/candle-trocr_candle_wrapper-tokenizer"))
            .get("tokenizer.json").unwrap();

        let tokenizer = Tokenizer::from_file(&tokenizer_file).map_err("ERROR OMG").unwrap();

        let mut tokenizer_dec = TokenOutputStream::new(tokenizer);

        let device = candle_examples::device(true).unwrap();

        let vb = {
            let model = {
                api.repo(hf_hub::Repo::with_revision(
                    "microsoft/trocr_candle_wrapper-base-handwritten".to_string(),
                    hf_hub::RepoType::Model,
                    "refs/pr/3".to_string(),
                ))
                    .get("model.safetensors").unwrap()
            };
            println!("model: {:?}", model);
            unsafe { VarBuilder::from_mmaped_safetensors(&[model], DType::F32, &device).unwrap() }
        };

        let (encoder_config, decoder_config) = {
            let config_filename = api
                .repo(hf_hub::Repo::with_revision(
                    "microsoft/trocr_candle_wrapper-base-handwritten".to_string(),
                    hf_hub::RepoType::Model,
                    "refs/pr/3".to_string(),
                ))
                .get("config.json").unwrap();
            let config: Config = serde_json::from_reader(std::fs::File::open(config_filename).unwrap()).unwrap();
            (config.encoder, config.decoder)
        };

        let mut m_model = trocr::TrOCRModel::new(&encoder_config, &decoder_config, vb).unwrap();

        Self {
            model: m_model
        }
    }

    // TODO: pub fn process_image (foo) -> String {...}

}

#[derive(Debug, Clone, serde::Deserialize)]
struct Config {
    encoder: vit::Config,
    decoder: trocr::TrOCRConfig,
}
