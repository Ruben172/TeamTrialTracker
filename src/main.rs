#[warn(clippy::pedantic)]
use plotly::Layout;
use plotly::{BoxPlot, Plot};
use plotly_static::{ImageFormat, StaticExporterBuilder};
use regex::Regex;
use rusty_tesseract::Args;
use rusty_tesseract::Image;
use rusty_tesseract::image::{GenericImageView, ImageReader};
use std::collections::HashMap;
use std::convert::Into;
use std::fs;
use std::path::{Path, PathBuf};

struct UmaData {
    name: String,
    scores: Vec<i32>,
}

impl UmaData {
    fn mean_score(&self) -> i32 {
        self.scores.iter().sum::<i32>() / self.scores.iter().count() as i32
    }
}

fn main() {
    let args = get_args();
    let score_regex = Regex::new(r"^([A-Za-z.']+(?: [A-Za-z.']+)*) (\d{1,3},\d{1,3})")
        .expect("Failed to compile regex");

    ensure_folder_exists("./input/");
    let input_files = fs::read_dir("./input/").expect("Failed to read input directory");
    let input_paths = input_files // AI written
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                if e.file_type().ok()?.is_file() {
                    Some(e.path())
                } else {
                    None
                }
            })
        })
        .collect::<Vec<_>>();

    let horse_names = fs::read_to_string("./uma.user-words")
        .unwrap_or("".to_string())
        .lines()
        .map(|x| x.to_string())
        .collect::<Vec<String>>();

    ensure_folder_exists("./output/");
    let mut scores: HashMap<String, Vec<i32>> = if fs::exists("./output/scores.json").unwrap() {
        serde_json::from_str(
            fs::read_to_string("./output/scores.json")
                .expect("json read failed")
                .as_str(),
        )
        .unwrap()
    } else {
        HashMap::<String, Vec<i32>>::new()
    };

    for file_path in &input_paths {
        let dynamic_image = ImageReader::open(&file_path)
            .expect("Failed to open image")
            .decode()
            .expect("Failed to decode image");
        let cropped_image = match dynamic_image.dimensions() {
            (1920, 1080) => dynamic_image.clone().crop(264, 88, 580, 855),
            (x, y) => {
                println!("Continuing with uncropped image ({}x{})", x, y);
                dynamic_image
            }
        };
        let image = Image::from_dynamic_image(&cropped_image).expect("Failed to convert image");
        let ocr_result = rusty_tesseract::image_to_string(&image, &args).expect("Tesseract error");

        for line in ocr_result.lines() {
            let Some(captures) = score_regex.captures(line) else {
                continue;
            };

            let name = captures[1]
                .split(" ")
                .filter(|x| horse_names.contains(&(x.to_string())))
                .collect::<Vec<&str>>()
                .join(" ");
            let score = match captures[2].split(",").collect::<String>().parse::<i32>() {
                Ok(s) => s,
                Err(_) => continue,
            };

            if !scores.contains_key(&name) {
                scores.insert(name.clone(), Vec::new());
            }

            scores.get_mut(&name).unwrap().push(score);
        }
    }

    let serialised = serde_json::to_string(&scores).unwrap();
    fs::write("./output/scores.json", serialised).expect("json write failed");

    ensure_folder_exists("./input/processed/");
    let processed_dir = PathBuf::from("./input/processed/");
    for file_path in &input_paths {
        let file_name = file_path
            .file_name()
            .expect("Couldn't find file name for image");
        let dest = processed_dir.join(file_name);
        if let Err(e) = fs::rename(&file_path, &dest) {
            eprintln!("Failed to move file {:?} to {:?}: {}", file_path, dest, e);
        }
    }

    let box_plot = make_box_plot(&scores);
    let mut exporter = StaticExporterBuilder::default()
        .build()
        .expect("Failed to build StaticExporter");

    exporter
        .write_fig(
            Path::new("./output/plot.png"),
            &serde_json::from_str(&box_plot.to_json()).expect("Failed to serialise boxplot"),
            ImageFormat::PNG,
            800,
            600,
            1.0,
        )
        .expect("Failed to export plot");

    exporter.close()
}

fn make_box_plot(scores: &HashMap<String, Vec<i32>>) -> Plot {
    let mut scores = scores
        .iter()
        .map(|(name, scores)| UmaData {
            name: name.clone(),
            scores: scores.clone(),
        })
        .collect::<Vec<UmaData>>();
    scores.sort_by(|x, y| x.mean_score().cmp(&y.mean_score()));

    let layout = Layout::new().show_legend(false).title("Team Trials scores");

    let mut plot = Plot::new();
    plot.set_layout(layout);
    for uma in scores {
        let trace = BoxPlot::new(uma.scores).name(uma.name);
        plot.add_trace(trace);
    }

    plot
}

fn ensure_folder_exists(path: &str) -> () {
    if !fs::exists(path).unwrap() {
        fs::create_dir_all(path).unwrap()
    }
}

fn get_args() -> Args {
    Args {
        lang: "eng".into(),
        config_variables: [
            (
                "tessedit_char_whitelist",
                "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890.,' ",
            ),
            ("user_patterns_file", "./uma.user-patterns"),
            ("user_words_file", "./uma.user-words"),
            ("load_system_dawg", "0"),
            ("load_freq_dawg", "0"),
        ]
        .iter()
        .cloned()
        .map(|(x, y)| (x.into(), y.into()))
        .collect(),

        dpi: Some(300),
        psm: Some(4),
        oem: Some(1),
    }
}
