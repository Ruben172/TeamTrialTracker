use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use regex::Regex;
use tesseract::{OcrEngineMode, PageSegMode, Tesseract};

use crate::{
    TESSDATA_DIR,
    image_helper::{crop_image, decode_image, dynamic_image_to_bytes},
};

pub fn ocr_image(path: &Path, mut tess: Tesseract) -> (String, Tesseract) {
    let image = decode_image(path);
    let crop = crop_image(image);
    tess = tess
        .set_image_from_mem(dynamic_image_to_bytes(&crop).as_slice()) // set_rectangle doesn't work well
        .expect("Failed to set image")
        // this might need configuring for other resolutions later, so far 300 has worked fine for all samples
        .set_source_resolution(300);
    (tess.get_text().expect("OCR Failed."), tess)
}

pub fn parse_orc_data(
    ocr_result: String,
    horse_names: &HashSet<String>,
    reg: &Regex,
    scores: &mut HashMap<String, Vec<u32>>,
) {
    for line in ocr_result.lines() {
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

pub fn setup_tesseract() -> Tesseract {
    let mut t = Tesseract::new_with_oem(Some(TESSDATA_DIR), Some("eng"), OcrEngineMode::LstmOnly)
        .expect("Failed to initialise OCR engine");
    t.set_page_seg_mode(PageSegMode::PsmSingleBlock);
    t = t
        .set_variable(
            "tessedit_char_whitelist",
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890.,' ",
        )
        .unwrap()
        .set_variable("user_patterns_file", "./tessdata/uma.user-patterns")
        .unwrap()
        .set_variable("user_words_file", "./tessdata/uma.user-words")
        .unwrap()
        .set_variable("load_system_dawg", "0")
        .unwrap()
        .set_variable("load_freq_dawg", "0")
        .unwrap();
    t
}
