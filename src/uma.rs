use crate::DATA_DIR;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub fn read_uma_colours() -> HashMap<String, String> {
    let mut colours_path = PathBuf::from(DATA_DIR);
    colours_path.push("uma_colours.json");
    let colours_file = fs::read_to_string(colours_path).expect("Failed to read colours file.");

    serde_json::from_str(&colours_file).expect("Failed to deserialize colours file.")
}
