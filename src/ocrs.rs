use image::GenericImageView;
use ocrs::{ImageSource, OcrEngine, OcrEngineParams};
use regex::Regex;
use rten::Model;
use std::path::PathBuf;
use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use crate::{
    OCR_DATA_DIR,
    image_helper::{crop_image, decode_image},
};

// Shortest possible is "????" (??????? characters)
const MIN_OCR_LENGTH: usize = 2;


pub fn ocr_image(path: &Path, engine: &OcrEngine) -> Vec<String> {
    let image = decode_image(path);
    let crop = crop_image(image);
    let image_source = ImageSource::from_bytes(crop.as_bytes(), crop.dimensions())
        .expect("Failed to create image source");
    let ocr_input = engine
        .prepare_input(image_source)
        .expect("Failed to prepare input");

    // engine.get_text(&ocr_input).expect("OCR failed.")

    let word_rects = engine
        .detect_words(&ocr_input)
        .expect("Failed to detect words.");
    let line_rects = engine.find_text_lines(&ocr_input, &word_rects);
    let line_texts = engine
        .recognize_text(&ocr_input, &line_rects)
        .expect("Failed to recognize text.");
    line_texts
        .iter()
        .flatten()
        .filter(|l| l.to_string().len() >= MIN_OCR_LENGTH)
        .map(|l| format!("{}", l))
        .collect()
}

pub fn parse_orc_data(
    ocr_result: Vec<String>,
    horse_names: &HashSet<String>,
    reg: &Regex,
    scores: &mut HashMap<String, Vec<u32>>,
) {
    for line in ocr_result.iter() {
        let Some(captures) = reg.captures(line) else {
            continue;
        };
        let name = captures[1]
            .split(' ')
            // discard all words from name matches that do not exist in any uma's name.
            .filter(|x| horse_names.contains(&(x.to_string())))
            .collect::<Vec<&str>>()
            .join(" ");
        let score = match captures[2].split(',').collect::<String>().parse::<u32>() {
            Ok(s) => s,
            Err(_) => continue,
        };

        if !scores.contains_key(&name) {
            scores.insert(name.clone(), Vec::new());
        }

        scores.get_mut(&name).unwrap().push(score);
    }
}

pub fn setup_engine() -> OcrEngine {
    let model_dir = PathBuf::from(OCR_DATA_DIR);
    let mut detection_model_path = model_dir.clone();
    detection_model_path.push("text-detection.rten");
    let mut recognition_model_path = model_dir.clone();
    recognition_model_path.push("text-recognition.rten");
    let detection_model = Model::load_file(detection_model_path).expect("Failed to load model");
    let recognition_model = Model::load_file(recognition_model_path).expect("Failed to load model");

    OcrEngine::new(OcrEngineParams {
        detection_model: Some(detection_model),
        recognition_model: Some(recognition_model),
        allowed_chars: Some(
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890.,' ".to_string(),
        ),
        ..Default::default()
    })
    .expect("Failed to make OCR Engine")
}
