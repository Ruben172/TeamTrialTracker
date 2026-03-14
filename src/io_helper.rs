use crate::{BACKUP_DIR, INPUT_DIR, OUTPUT_DIR, OUTPUT_FILE};
use chrono::{DateTime, Datelike, Local, Timelike};
use std::{collections::HashMap, fs, path::PathBuf};

pub fn read_input_dir() -> Vec<PathBuf> {
    ensure_folder_exists(INPUT_DIR);
    let input_files = fs::read_dir(INPUT_DIR)
        .expect(&format!("Failed to read input directory : \"{INPUT_DIR}\""));
    input_files // AI written
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                if e.file_type().ok()?.is_file() {
                    Some(e.path())
                } else {
                    None
                }
            })
        })
        .collect::<Vec<_>>()
}

pub fn read_scores() -> HashMap<String, Vec<u32>> {
    ensure_folder_exists(OUTPUT_DIR);
    if fs::exists(OUTPUT_FILE).unwrap() {
        serde_json::from_str(
            fs::read_to_string(OUTPUT_FILE)
                .expect("output_file read failed")
                .as_str(),
        )
        .expect("json parse of output_file failed")
    } else {
        HashMap::<String, Vec<u32>>::new()
    }
}

pub fn save_scores(scores: &HashMap<String, Vec<u32>>, input_paths: &Vec<PathBuf>) {
    backup_old_scores();

    let serialised = serde_json::to_string_pretty(&scores).unwrap();
    fs::write(OUTPUT_FILE, serialised).expect("output_file write failed");
    move_all_files(input_paths, "./input/processed");
}

fn backup_old_scores() {
    if !fs::exists(OUTPUT_FILE).unwrap() {
        return;
    }

    ensure_folder_exists(BACKUP_DIR);
    let dt: DateTime<Local> = fs::metadata(OUTPUT_FILE)
        .expect("Failed to read output metadata")
        .modified()
        .expect("Failed to read output creation time for backup")
        .into();

    let dest = PathBuf::from(BACKUP_DIR).join(format!(
        "{:02}-{:02}-{:02} {:02}{:02}{:02}",
        dt.year(),
        dt.month(),
        dt.day(),
        dt.hour(),
        dt.minute(),
        dt.second()
    ));
    fs::rename(OUTPUT_FILE, dest).expect("Failed to move scores to backup destination")
}

fn move_all_files(input_paths: &Vec<PathBuf>, dest_path: &str) {
    ensure_folder_exists(dest_path);
    let dest_dir = PathBuf::from(dest_path);
    for file_path in input_paths {
        let file_name = file_path
            .file_name()
            .expect("Couldn't find file name for image");
        let dest = dest_dir.join(file_name);
        if let Err(e) = fs::rename(file_path, &dest) {
            eprintln!("Failed to move file {:?} to {:?}: {}", file_path, dest, e);
        }
    }
}

fn ensure_folder_exists(path: &str) {
    if !fs::exists(path).unwrap() {
        fs::create_dir_all(path).unwrap()
    }
}
