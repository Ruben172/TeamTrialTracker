use BoxPlotType::*;
use image::{Rgb, RgbImage};
use std::{collections::HashMap, fmt::Display};
use image::imageops::overlay;

const IMAGE_BORDER_WIDTH: u32 = 80;
const BOX_WIDTH: u32 = 42;
const IMAGE_HEIGHT: u32 = 600;
const PLOTS_AREA_HEIGHT: u32 = 400;

struct UmaBoxPlot {
    label: String,
    min: u32,
    median: u32,
    mean: u32,
    max: u32,
    q1: u32,
    q3: u32,
    lower_whisker: u32,
    upper_whisker: u32,
    outliers: Vec<u32>,
}

/// Data must be sorted and fraction must be between 0-1
fn get_percentile(data: &Vec<u32>, fraction: f64) -> u32 {
    let count = (data.len() - 1) as f64;
    let point = count * fraction;
    let lower_idx = point.floor() as usize;
    let upper_idx = point.ceil() as usize;

    let lower_val = data[lower_idx] as f64;
    let upper_val = data[upper_idx] as f64;
    let fract = point.fract();

    (lower_val * (1.0 - fract) + upper_val * fract) as u32
}

impl From<&UmaData> for UmaBoxPlot {
    fn from(uma_data: &UmaData) -> Self {
        let mut sorted_scores = uma_data.scores.clone();
        sorted_scores.sort_unstable();
        let mean = sorted_scores.iter().sum::<u32>() / sorted_scores.len() as u32;

        let min_score = *sorted_scores.first().unwrap();
        let max_score = *sorted_scores.last().unwrap();

        let q1 = get_percentile(&sorted_scores, 0.25) as f64;
        let median = get_percentile(&sorted_scores, 0.5);
        let q3 = get_percentile(&sorted_scores, 0.75) as f64;

        let iqr = q3 - q1;
        let lower_whisker = ((q1 - 1.5 * iqr) as u32).max(min_score);
        let upper_whisker = ((q3 + 1.5 * iqr) as u32).min(max_score);

        let outliers: Vec<u32> = sorted_scores
            .into_iter()
            .filter(|s| s < &lower_whisker || s > &upper_whisker)
            .collect();

        UmaBoxPlot {
            label: uma_data.name.clone(),
            min: min_score,
            median,
            mean,
            max: max_score,
            q1: q1 as u32,
            q3: q3 as u32,
            lower_whisker,
            upper_whisker,
            outliers,
        }
    }
}

pub fn render_plots(umadata: Vec<UmaData>, uma_colours: &HashMap<String, String>) {
    let boxplot_data: Vec<UmaBoxPlot> = umadata
        .iter()
        .filter(|u| u.scores.len() > 0)
        .map(|u| UmaBoxPlot::from(u))
        .collect();

    let mut base_image = make_base_image(boxplot_data.len());

    todo!()
}

fn make_base_image(character_slots: usize) -> RgbImage {
    let width = 2 * IMAGE_BORDER_WIDTH + BOX_WIDTH * (character_slots as u32);

    RgbImage::from_pixel(width, IMAGE_HEIGHT, Rgb([255, 255, 255]))
}

pub struct UmaData {
    pub name: String,
    pub scores: Vec<u32>,
}

impl UmaData {
    pub fn mean_score(&self) -> u32 {
        self.scores.iter().sum::<u32>() / self.scores.len() as u32
    }

    pub fn median_score(&self) -> u32 {
        let mut sorted_scores = self.scores.clone();
        sorted_scores.sort_unstable();
        let mid = sorted_scores.len() / 2;
        if sorted_scores.len().is_multiple_of(2) {
            (sorted_scores[mid - 1] + sorted_scores[mid]) / 2
        } else {
            sorted_scores[mid]
        }
    }

    pub fn from_scores(scores: &HashMap<String, Vec<u32>>) -> Vec<UmaData> {
        scores
            .iter()
            .map(|(name, scores)| UmaData {
                name: name.clone(),
                scores: scores.clone(),
            })
            .collect::<Vec<UmaData>>()
    }
}

enum BoxPlotType {
    Min,
    Mean,
    Median,
    Max,
}

impl BoxPlotType {
    fn to_comparer(&self) -> fn(&UmaData) -> u32 {
        match self {
            Min => |x| *x.scores.iter().min().unwrap(),
            Mean => UmaData::mean_score,
            Median => UmaData::median_score,
            Max => |x| *x.scores.iter().max().unwrap(),
        }
    }

    fn to_file_name(&self) -> &str {
        match self {
            Min => "min",
            Mean => "mean",
            Median => "median",
            Max => "max",
        }
    }
}

impl Display for BoxPlotType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Min => "Minimum",
            Mean => "Mean",
            Median => "Median",
            Max => "Maximum",
        };
        write!(f, "{}", str)
    }
}

// AI
fn hex_to_rgb(hex: &str) -> Option<(u8, u8, u8)> {
    let h = hex.trim_start_matches('#');
    if h.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&h[0..2], 16).ok()?;
    let g = u8::from_str_radix(&h[2..4], 16).ok()?;
    let b = u8::from_str_radix(&h[4..6], 16).ok()?;
    Some((r, g, b))
}

// AI
fn rgb_to_hex(r: u8, g: u8, b: u8) -> String {
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

// AI
fn darken_hex(hex: &String) -> String {
    // factor in (0.0, 1.0) where 1.0 = same colour, 0.8 = 20% darker
    let factor = 0.7;
    if let Some((r, g, b)) = hex_to_rgb(hex) {
        let rf = ((r as f32) * factor).round().clamp(0.0, 255.0) as u8;
        let gf = ((g as f32) * factor).round().clamp(0.0, 255.0) as u8;
        let bf = ((b as f32) * factor).round().clamp(0.0, 255.0) as u8;
        rgb_to_hex(rf, gf, bf)
    } else {
        // if input not valid hex, return it unchanged
        hex.to_string()
    }
}
