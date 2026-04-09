#[warn(clippy::pedantic)]
mod image_helper;
mod io_helper;
mod plot;
mod ocrs;
mod uma;

use io_helper::{read_input_dir, read_scores, save_scores};
use plot::{create_plots, render_plots, UmaData};
use regex::Regex;
use ocrs::{ocr_image, parse_orc_data, setup_engine};
use uma::read_uma_names;

const INPUT_DIR: &str = "./input/";
const OUTPUT_DIR: &str = "./output/";
const OUTPUT_FILE: &str = "./output/scores.json";
const BACKUP_DIR: &str = "./output/backup/";
const OCR_DATA_DIR: &str = "./.ocr_data/";
const UMA_NAME_FILE: &str = "./.tessdata/uma_words";

fn main() {
    // capture name, then capture score (comma included), has (?:.* )? in the middle in case MVP gets OCRed as bogus text
    let score_regex = Regex::new(r"^([A-Za-z.']+(?: [A-Za-z.']+)*) (?:.* )?(\d{1,3},\d{1,3})")
        .expect("Failed to compile regex");

    let input_paths = read_input_dir();

    let horse_names = read_uma_names();

    let mut scores = read_scores();

    println!("Starting OCR...");
    let engine = setup_engine();
    for file_path in &input_paths {
        // images
        let ocr_result = ocr_image(file_path, &engine);
        parse_orc_data(ocr_result, &horse_names, &score_regex, &mut scores);
    }

    save_scores(&scores, &input_paths);

    let mut umadata = UmaData::from_scores(&scores);
    let plots = create_plots(&mut umadata);
    render_plots(plots);
}
