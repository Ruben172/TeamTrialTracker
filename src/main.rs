#[warn(clippy::pedantic)]
use regex::Regex;

mod image_helper;
mod io_helper;
mod plot;
mod tess;
mod uma;
use io_helper::{read_input_dir, read_output, write_output};
use plot::create_box_plot;
use tess::setup_tesseract;
use uma::read_uma_names;

use crate::tess::{ocr_image, parse_orc_data};

const INPUT_DIR: &str = "./input/";
const OUTPUT_DIR: &str = "./output/";
const OUTPUT_FILE: &str = "./output/scores.json";
const TESSDATA_DIR: &str = "./tessdata/";
const UMA_NAME_FILE: &str = "./tessdata/uma.user-words";

fn main() {
    // capture name, then capture score (comma included), has (?:.* )? in the middle in case MVP gets OCRed as bogus text
    let score_regex = Regex::new(r"^([A-Za-z.']+(?: [A-Za-z.']+)*) (?:.* )?(\d{1,3},\d{1,3})")
        .expect("Failed to compile regex");

    let input_paths = read_input_dir();

    let horse_names = read_uma_names();

    let mut scores = read_output();

    let mut tesseract = setup_tesseract();

    println!("Starting OCR...");
    for file_path in &input_paths {
        let ocr_result: String;
        (ocr_result, tesseract) = ocr_image(file_path, tesseract);
        parse_orc_data(ocr_result, &horse_names, &score_regex, &mut scores);
    }

    write_output(&scores, &input_paths);

    create_box_plot(&scores);
}
