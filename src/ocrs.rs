use crate::{
    DATA_DIR,
    image_helper::{crop_image, decode_image},
};
use image::GenericImageView;
use ocrs::{ImageSource, OcrEngine, OcrEngineParams};
use rten::Model;
use std::path::PathBuf;
use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

const MIN_OCR_LENGTH: usize = 3;

pub fn ocr_image(path: &Path, engine: &OcrEngine) -> Vec<String> {
    let image = decode_image(path);
    let crop = crop_image(image);
    let image_source = ImageSource::from_bytes(crop.as_bytes(), crop.dimensions())
        .expect("Failed to create image source");
    let ocr_input = engine
        .prepare_input(image_source)
        .expect("Failed to prepare input");

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

struct OcrParseResult {
    ocr_name: String,
    corrected_name: String,
    order: usize,
    distance: usize,
}

/// Results often look like
/// "Mayano Top Gun"
/// "47,131 pts i"
///
/// This may be interrupted by an MVP text, or the OCR engine will first read all names before
/// reading scores. Epithets will also often show up.
///
/// Unsure if the scores can also be in a single line, but this parsing function does account for it.
pub fn parse_orc_data(
    ocr_result: Vec<String>,
    uma_names: &HashSet<String>,
    file_path: &PathBuf,
) -> HashMap<String, u32> {
    let mut names: Vec<OcrParseResult> = Vec::new();
    let mut scores: Vec<u32> = Vec::new();
    let mut order = 0;

    for line in ocr_result.iter() {
        let mut text = String::new();
        let mut numbers = String::new();

        let mut seen_digits = 0;
        for char in line.chars() {
            // Ignore commas
            if char == ',' {
                continue;
            }
            // Use digits to differentiate between names (or other text) and scores
            match char.is_ascii_digit() {
                true => {
                    seen_digits += 1;
                    numbers.push(char);
                }
                false => {
                    // If we've seen at least four digits and approach a non-digit, we reached the end of a score.
                    if seen_digits > 3 {
                        break;
                    }
                    text.push(char);
                }
            }
        }

        // Remove any "MVP" strings that might be attached to names. Trim leading/trailing whitespace.
        text = text.split("MVP").collect::<String>().trim().to_string();
        // For every text entry, add the closest uma name to the list of names.
        if text.len() > 3 {
            let mut closest_name = "Mambo";
            let mut lowest_distance = 1000;
            order += 1;

            for uma_name in uma_names {
                let distance = strsim::levenshtein(uma_name, &text);
                if distance < lowest_distance {
                    closest_name = uma_name;
                    lowest_distance = distance;
                }
            }

            names.push(OcrParseResult {
                ocr_name: text,
                corrected_name: closest_name.to_string(),
                order,
                distance: lowest_distance,
            });
        }

        // Is a score below 1000 even possible?
        if numbers.len() > 3 {
            scores.push(numbers.parse::<u32>().expect("Failed to parse score."))
        }
    }

    let scores_count = scores.len();
    names.sort_by_key(|name| name.distance);
    let mut closest_names: Vec<OcrParseResult> = names.into_iter().take(scores_count).collect();
    closest_names.sort_by_key(|name| name.order);

    let mut res = HashMap::new();
    for (i, result) in closest_names.iter().enumerate() {
        if result.ocr_name != result.corrected_name {
            let path_str = file_path
                .to_str()
                .expect("Failed to convert path to string");
            println!(
                "{}\nCorrected incorrectly read name: {} -> {}. Distance: {}",
                path_str, result.ocr_name, result.corrected_name, result.distance
            )
        }

        res.insert(result.corrected_name.clone(), scores[i]);
    }

    res
}

pub fn setup_engine() -> OcrEngine {
    let model_dir = PathBuf::from(DATA_DIR);
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
