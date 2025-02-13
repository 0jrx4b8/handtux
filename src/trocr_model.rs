use crate::trocr_processor::ImageProcessor;
use candle_core::{DType, Device, Error, Tensor};
use candle_examples::token_output_stream::TokenOutputStream;
use candle_nn::VarBuilder;
use candle_transformers::models::{trocr, vit};
use tokenizers::Tokenizer;
use tokio::sync::mpsc::Sender;

pub struct TrOCRImplementationHandtux {
    processor: ImageProcessor,
    device: Device,
    model: Option<trocr::TrOCRModel>,
    config: Option<Config>,
    tokenizer: Option<Tokenizer>,
    //tokenizer_dec: Option<TokenOutputStream>,
    data_tx: Option<Sender<Vec<String>>>,
    status_tx: Option<Sender<char>>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct Config {
    encoder: vit::Config,
    decoder: trocr::TrOCRConfig,
}

impl TrOCRImplementationHandtux {
    pub fn new() -> Self {
        Self {
            processor: ImageProcessor::new(),
            device: Device::cuda_if_available(0).unwrap(),
            model: None,
            config: None,
            tokenizer: None,
            //tokenizer_dec: None,
            data_tx: None,
            status_tx: None,
        }
    }

    pub async fn init(&mut self, data_tx: Sender<Vec<String>>, status_tx: Sender<char>) {
        let api = hf_hub::api::sync::Api::new().unwrap();
        let tokenizer_path = api
            .model(String::from("ToluClassics/candle-trocr-tokenizer"))
            .get("tokenizer.json")
            .unwrap();
        let tokenizer = Tokenizer::from_file(tokenizer_path.clone()).unwrap();
        //let tokenizer_dec = TokenOutputStream::new(Tokenizer::from_file(tokenizer_path).unwrap());

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

        status_tx.send('R').await.unwrap();

        self.model = Some(model);
        self.config = Some(config);
        self.tokenizer = Some(tokenizer);
        //self.tokenizer_dec = Some(tokenizer_dec);
        self.data_tx = Some(data_tx);
        self.status_tx = Some(status_tx);
        // Self {
        //     processor: ImageProcessor::new(),
        //     device,
        //     model,
        //     config,
        //     tokenizer_dec,
        //     data_tx,
        //     status_tx,
        // }
    }

    pub async fn get_candidates(
        &mut self,
        painting_frame: &Vec<[eframe::egui::Pos2; 2]>,
    ) -> Result<Vec<String>, Error> {
        let mut m_tokenizer_dec = TokenOutputStream::new(self.tokenizer.clone().unwrap());
        self.status_tx.as_ref().unwrap().send('T').await.unwrap();
        let mut result_text = Vec::new();

        println!("Preprocessing image...");
        let preprocessed = self
            .processor
            .preprocess(painting_frame)?
            .unsqueeze(0)?
            .to_device(&self.device)?;
        println!("Image preprocessed!");

        // Encode the image
        println!("Encoding image...");
        let encoder_xs = self
            .model
            .as_mut()
            .unwrap()
            .encoder()
            .forward(&preprocessed)?;
        println!("Image encoded!");

        // Generate the text
        println!("Generating text...");
        let mut logits_processor =
            candle_transformers::generation::LogitsProcessor::new(1337, None, None);
        let mut token_ids = vec![self
            .config
            .as_ref()
            .unwrap()
            .decoder
            .decoder_start_token_id
            .clone()];
        for index in 0..128 {
            let context_size = if index >= 1 { 1 } else { token_ids.len() };
            let start_pos = token_ids.len().saturating_sub(context_size);
            let input_ids = Tensor::new(&token_ids[start_pos..], &self.device)?.unsqueeze(0)?;
            let logits = self
                .model
                .as_mut()
                .unwrap()
                .decode(&input_ids, &encoder_xs, start_pos)?;
            let logits = logits.squeeze(0)?;
            let logits = logits.get(logits.dim(0)? - 1)?;
            let token = logits_processor.sample(&logits)?;
            token_ids.push(token);

            if let Some(t) = m_tokenizer_dec.next_token(token)? {
                //use std::io::Write;
                //print!("{t}");
                result_text.push(t);
                //std::io::stdout().flush()?;
            }
            if token == self.config.as_ref().unwrap().decoder.eos_token_id {
                break;
            }
        }
        println!("Text generated!");

        self.data_tx
            .as_ref()
            .unwrap()
            .send(result_text.clone())
            .await
            .unwrap();

        Ok(vec!["Grah".to_string(), "Grah".to_string()])
    }
}
