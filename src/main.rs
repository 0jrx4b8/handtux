// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use std::io::Cursor;
use image::ImageDecoder;
use kalosm::vision::*;
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
async fn perform_ocr(state: State<'_, ModelState>, data_url: String) -> Result<String, String> {
    let mut model = state.0.lock().map_err(|_| "Err unlocking").unwrap();

    let base64_data = data_url.split(',').nth(1).ok_or("Invalid data URL format").unwrap();
    let image_base64_data = BASE64_STANDARD.decode(base64_data).map_err(|_| "Failed to decode image data").unwrap();
    let d_image =
        image::io::Reader::new(Cursor::new(image_base64_data))
            .with_guessed_format()
            .unwrap()
            .decode()
            .unwrap();

    let inference_settings = OcrInferenceSettings::new(d_image).unwrap();

    let output = model
        .recognize_text(inference_settings)
        .unwrap();

    println!("{}", output);

    Ok(output)
}

struct ModelState(Mutex<Ocr>);

#[tokio::main]
async fn main() {
    let model = Ocr::builder().build().await.unwrap();

    let model_state = ModelState(Mutex::new(model));

    tauri::Builder::default()
        .manage(model_state)
        .invoke_handler(tauri::generate_handler![perform_ocr])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
