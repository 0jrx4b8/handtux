// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashMap;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use std::io::Cursor;
use image::ImageReader;
use rusty_tesseract::{Args, Image};

#[tauri::command]
async fn perform_ocr(data_url: String) -> String {
    let base64_data = data_url.split(',').nth(1).ok_or("Invalid data URL format").unwrap();
    let image_base64_data = BASE64_STANDARD.decode(base64_data).map_err(|_| "Failed to decode image data").unwrap();
    let image_reader =
        ImageReader::new(Cursor::new(image_base64_data))
            .with_guessed_format()
            .unwrap()
            .decode()
            .unwrap();
    let image = Image::from_dynamic_image(&image_reader).unwrap();

    let default_args = Args {
        lang: "osd".to_string() ,
        config_variables: HashMap::from([(
            "tessedit_char_whitelist".into(),
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789@.;?:/!+=-$£€* '".into(),
        )]),
        dpi:Some(600),
        psm:Some(11),
        oem:Some(1)
    };

    let output = rusty_tesseract::image_to_string(&image, &default_args).unwrap();

    println!("The string is: {:?}", output);

    output
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![perform_ocr])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
