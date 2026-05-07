use BoxPlotType::*;
use image::imageops::overlay;
use image::{Rgba, RgbaImage};
use imageproc::drawing::draw_filled_rect_mut;
use imageproc::rect::Rect;
use palette::{Darken, Srgb, Srgba, WithAlpha};
use std::collections::HashSet;
use std::str::FromStr;
use std::{collections::HashMap, fmt::Display};

const IMAGE_BORDER_WIDTH: u32 = 80;
const BOXPLOT_CONTAINER_WIDTH: u32 = 42;
const IQR_WIDTH: u32 = 22;
const BOX_BORDER_WIDTH: u32 = 2;
const WHISKER_WIDTH: u32 = 2;
const WHISKER_CAP_WIDTH: u32 = 10;
const WHISKER_CAP_HEIGHT: u32 = 2;
const OUTLIER_WIDTH: u32 = 6;
const IMAGE_HEIGHT: u32 = 600;
const BOXPLOT_CONTAINER_HEIGHT: u32 = 400;
const SCORES_MARGIN: f64 = 0.05;

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
    outliers: HashSet<u32>,
}

struct Bounds {
    upper: u32,
    lower: u32,
    delta: u32,
}

impl Bounds {
    fn new(min: u32, max: u32) -> Self {
        let margin = ((max - min) as f64 * SCORES_MARGIN) as u32;
        let upper = max + margin;
        let lower = min - margin;

        Bounds {
            upper,
            lower,
            delta: upper - lower,
        }
    }
}

/// Data must be sorted and fraction must be between 0-1
fn get_percentile(data: &[u32], fraction: f64) -> u32 {
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
        let lower_bound = ((q1 - 1.5 * iqr) as u32).max(min_score);
        let upper_bound = ((q3 + 1.5 * iqr) as u32).min(max_score);

        let outliers: HashSet<u32> = sorted_scores
            .clone()
            .into_iter()
            .filter(|s| s < &lower_bound || s > &upper_bound)
            .collect();

        let lower_whisker = *(sorted_scores
            .iter()
            .find(|s| !outliers.contains(s))
            .unwrap());
        let upper_whisker = *(sorted_scores
            .iter()
            .rfind(|s| !outliers.contains(s))
            .unwrap());

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
    let mut boxplot_data: Vec<UmaBoxPlot> = umadata
        .iter()
        .filter(|u| !u.scores.is_empty())
        .map(UmaBoxPlot::from)
        .collect();
    boxplot_data.sort_by_key(|data| data.median);

    let min_score = boxplot_data.iter().fold(u32::MAX, |acc, x| acc.min(x.min));
    let max_score = boxplot_data.iter().fold(u32::MIN, |acc, x| acc.max(x.max));
    let bounds = Bounds::new(min_score, max_score);

    let mut image = make_base_image(boxplot_data.len());

    let y = (IMAGE_HEIGHT - BOXPLOT_CONTAINER_HEIGHT) as i64 / 2;
    for (i, uma_data) in boxplot_data.iter().enumerate() {
        let name = &uma_data.label;
        let box_base_colour = Rgba(
            uma_colours
                .get(name)
                .map(|s| {
                    Srgb::from_str(s)
                        .unwrap_or_else(|_| panic!("Failed to parse colour for {name}"))
                        .with_alpha(255)
                        .into()
                })
                .unwrap_or([0, 0, 0, 255]), // black
        );
        let uma_box = render_boxplot(uma_data, &bounds, box_base_colour);
        // uma_box.save(format!("tests/{name}.png")).unwrap();

        let x = (IMAGE_BORDER_WIDTH + (BOXPLOT_CONTAINER_WIDTH * i as u32)) as i64;
        overlay(&mut image, &uma_box, x, y);
    }

    image.save("test.png").unwrap();

    todo!()
}

fn make_base_image(character_slots: usize) -> RgbaImage {
    let width = 2 * IMAGE_BORDER_WIDTH + BOXPLOT_CONTAINER_WIDTH * (character_slots as u32);

    RgbaImage::from_pixel(width, IMAGE_HEIGHT, Rgba([255, 255, 255, 255]))
}

fn render_boxplot(uma_data: &UmaBoxPlot, bounds: &Bounds, base_colour: Rgba<u8>) -> RgbaImage {
    let mut base = RgbaImage::new(BOXPLOT_CONTAINER_WIDTH, BOXPLOT_CONTAINER_HEIGHT);

    let mut inner_colour = base_colour;
    inner_colour[3] = 204;
    let border_colour: Rgba<u8> = Rgba(
        Srgba::from(base_colour.0)
            .into_format()
            .darken_fixed(0.15)
            .into_format()
            .into(),
    );

    draw_whiskers(&mut base, uma_data, bounds, border_colour);
    draw_iqr(&mut base, uma_data, bounds, inner_colour, border_colour);
    draw_median(&mut base, uma_data.median, bounds, border_colour);

    for outlier in &uma_data.outliers {
        draw_outlier(&mut base, *outlier, bounds, border_colour);
    }

    base
}

fn draw_whiskers(base: &mut RgbaImage, uma_data: &UmaBoxPlot, bounds: &Bounds, colour: Rgba<u8>) {
    let top_whisker = get_score_y(uma_data.upper_whisker, bounds, BOXPLOT_CONTAINER_HEIGHT).floor();
    let bot_whisker = get_score_y(uma_data.lower_whisker, bounds, BOXPLOT_CONTAINER_HEIGHT).floor();

    // Whisker
    let whisker_start_x = ((BOXPLOT_CONTAINER_WIDTH - WHISKER_WIDTH) / 2) as i32;
    let whisker_rect = Rect::at(whisker_start_x, top_whisker as i32)
        .of_size(WHISKER_WIDTH, (bot_whisker - top_whisker) as u32);
    draw_filled_rect_mut(base, whisker_rect, colour);

    // Whisker cap
    let whisker_cap_start_x = ((BOXPLOT_CONTAINER_WIDTH - WHISKER_CAP_WIDTH) / 2) as i32;
    let top_cap_rect = Rect::at(whisker_cap_start_x, top_whisker as i32)
        .of_size(WHISKER_CAP_WIDTH, WHISKER_CAP_HEIGHT);
    let bot_cap_rect = Rect::at(whisker_cap_start_x, bot_whisker as i32)
        .of_size(WHISKER_CAP_WIDTH, WHISKER_CAP_HEIGHT);
    draw_filled_rect_mut(base, top_cap_rect, colour);
    draw_filled_rect_mut(base, bot_cap_rect, colour);
}

fn draw_iqr(
    base: &mut RgbaImage,
    uma_data: &UmaBoxPlot,
    bounds: &Bounds,
    base_colour: Rgba<u8>,
    border_colour: Rgba<u8>,
) {
    let box_top = get_score_y(uma_data.q3, bounds, BOXPLOT_CONTAINER_HEIGHT) as i32;
    let box_bottom = get_score_y(uma_data.q1, bounds, BOXPLOT_CONTAINER_HEIGHT) as i32;
    let box_height = (box_bottom - box_top) as u32;

    // Border
    let iqr_start_x = ((BOXPLOT_CONTAINER_WIDTH - IQR_WIDTH) / 2) as i32;
    let outline_rect = Rect::at(iqr_start_x, box_top).of_size(IQR_WIDTH, box_height);
    draw_filled_rect_mut(base, outline_rect, border_colour);

    // Inside
    let inner_rect = Rect::at(
        iqr_start_x + BOX_BORDER_WIDTH as i32,
        box_top + BOX_BORDER_WIDTH as i32,
    )
    .of_size(
        IQR_WIDTH - BOX_BORDER_WIDTH * 2,
        box_height - BOX_BORDER_WIDTH * 2,
    );
    draw_filled_rect_mut(base, inner_rect, base_colour);
}

fn draw_median(base: &mut RgbaImage, median: u32, bounds: &Bounds, colour: Rgba<u8>) {
    let median_y = get_score_y(median, bounds, BOXPLOT_CONTAINER_HEIGHT).floor() as i32;

    let iqr_start_x = ((BOXPLOT_CONTAINER_WIDTH - IQR_WIDTH) / 2) as i32;
    let median_rect = Rect::at(iqr_start_x, median_y).of_size(IQR_WIDTH, 2);
    draw_filled_rect_mut(base, median_rect, colour);
}

// Manually draws a circle with multiple centre pixels, won't work if OUTLIER_WIDTH is changed.
fn draw_outlier(base: &mut RgbaImage, outlier: u32, bounds: &Bounds, colour: Rgba<u8>) {
    let outlier_start_x = (BOXPLOT_CONTAINER_WIDTH - OUTLIER_WIDTH) as i32 / 2;
    let outlier_start_y = get_score_y(outlier, bounds, BOXPLOT_CONTAINER_HEIGHT).floor() as i32
        - (OUTLIER_WIDTH as i32 / 2);

    let horizontal = Rect::at(outlier_start_x, outlier_start_y + 2).of_size(OUTLIER_WIDTH, 2);
    let middle = Rect::at(outlier_start_x + 1, outlier_start_y + 1).of_size(4, 4);
    let vertical = Rect::at(outlier_start_x + 2, outlier_start_y).of_size(2, OUTLIER_WIDTH);
    draw_filled_rect_mut(base, horizontal, colour);
    draw_filled_rect_mut(base, middle, colour);
    draw_filled_rect_mut(base, vertical, colour);
}

fn get_score_y(score: u32, bounds: &Bounds, image_height: u32) -> f32 {
    let fraction_from_bottom = (score - bounds.lower) as f32 / bounds.delta as f32;
    let y_from_bottom = fraction_from_bottom * image_height as f32;
    image_height as f32 - y_from_bottom
}

pub struct UmaData {
    pub name: String,
    pub scores: Vec<u32>,
}

impl UmaData {
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
