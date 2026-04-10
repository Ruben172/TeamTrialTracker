#[warn(clippy::pedantic)]
mod image_helper;
mod io_helper;
mod ocr;
mod plot;
mod uma;

use io_helper::{read_input_dir, read_scores, save_scores};
use ocr::{ocr_image, parse_orc_data, setup_engine};
use plot::{UmaData, create_plots, render_plots};
use std::collections::HashMap;
use uma::read_uma_colours;

const INPUT_DIR: &str = "./input/";
const OUTPUT_DIR: &str = "./output/";
const OUTPUT_FILE: &str = "./output/scores.json";
const BACKUP_DIR: &str = "./output/backup/";
const DATA_DIR: &str = "./.data/";

fn main() {
    let input_paths = read_input_dir();

    let uma_colours = read_uma_colours();
    let uma_names = uma_colours
        .keys()
        .cloned()
        .collect::<std::collections::HashSet<String>>();

    let mut new_scores: Vec<HashMap<String, u32>> = Vec::new();

    println!("Starting OCR...");
    let engine = setup_engine();
    for file_path in &input_paths {
        // images
        let ocr_result = ocr_image(file_path, &engine);
        let new_score = parse_orc_data(ocr_result, &uma_names, file_path);
        new_scores.push(new_score);
    }

    let scores = read_scores();
    let combined_scores = new_scores.iter().fold(scores, |mut acc, new_score| {
        for (key, value) in new_score.clone() {
            acc.entry(key)
                .and_modify(|vec| vec.push(value))
                .or_insert(vec![value]);
        }
        acc
    });
    save_scores(&combined_scores, &input_paths);

    let mut umadata = UmaData::from_scores(&combined_scores);
    let plots = create_plots(&mut umadata, &uma_colours);
    render_plots(plots);
}
