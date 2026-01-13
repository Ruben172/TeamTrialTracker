use crate::{OUTPUT_DIR, uma::uma_fill_color};
use BoxPlotType::*;
use itertools::Itertools;
use plotters::{
    chart::{ChartBuilder, SeriesLabelPosition},
    coord::Shift,
    data::{Quartiles, fitting_range},
    prelude::{
        BitMapBackend, Boxplot, DrawingArea, DrawingBackend, IntoDrawingArea, IntoSegmentedCoord,
        Rectangle, SVGBackend, SegmentValue,
    },
    style::{BLACK, Color, RGBAColor, RGBColor, ShapeStyle, WHITE},
};
use std::{
    collections::{BTreeMap, HashMap},
    env,
    fmt::Display,
    path::{Path, PathBuf},
};

pub fn create_box_plot(scores: &HashMap<String, Vec<u32>>) {
    let name_min = format!("{OUTPUT_DIR}min.svg");
    let name_mean = format!("{OUTPUT_DIR}mean.svg");
    let name_max = format!("{OUTPUT_DIR}max.svg");
    let mut min = write_box_plot(&name_min).into_drawing_area();
    let mut mean = write_box_plot(&name_mean).into_drawing_area();
    let mut max = write_box_plot(&name_max).into_drawing_area();
    make_box_plot(&scores, Min, &mut min);
    make_box_plot(&scores, Mean, &mut mean);
    make_box_plot(&scores, Max, &mut max);

    println!(
        "Rendering box plots... do not close the application (or you will have to manually kill geckodriver)"
    );
    min.present();
    mean.present();
    max.present();
}

fn make_box_plot<B: DrawingBackend>(
    scores: &HashMap<String, Vec<u32>>,
    box_plot_type: BoxPlotType,
    canvas: &mut DrawingArea<B, Shift>,
) {
    let comparer = box_plot_type.to_comparer();
    let mut scores = scores
        .iter()
        .map(|(name, scores)| UmaData {
            name: name.clone(),
            scores: scores.clone(),
        })
        .collect::<Vec<UmaData>>();
    scores.sort_by(|x, y| comparer(x).cmp(&comparer(&y)));

    let shown_score: u32 = scores.iter().map(|x| comparer(x)).sum();
    let values: Vec<u32> = scores.iter().flat_map(|x| x.scores.clone()).collect();
    let values_range = fitting_range(values.iter());

    let umas = scores.iter().map(|u| u.name.clone()).collect::<Vec<_>>();

    let dataset: Vec<(String, Quartiles)> = scores
        .iter()
        .map(|UmaData { name, scores }| (name.clone(), Quartiles::new(scores)))
        .collect();

    canvas.fill(&WHITE).unwrap();

    let mut offsets = (-12..).step_by(24);
    let mut series = BTreeMap::new();
    for (name, quart) in dataset.iter() {
        let color = hex_to_rgb(&uma_fill_color(&name)).unwrap();
        let color = RGBColor(color.0, color.1, color.2).filled();
        let entry = series
            .entry(name.clone())
            .or_insert_with(|| (Vec::new(), color, offsets.next().unwrap()));
        entry.0.push((name.clone(), quart));
    }

    let canvas = canvas.margin(5, 5, 5, 5);

    let mut chart = ChartBuilder::on(&canvas)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .caption(
            format!(
                "{} Team Trial score: {}",
                box_plot_type.to_string(),
                shown_score
            ),
            ("sans-serif", 20),
        )
        .build_cartesian_2d(umas[..].into_segmented(), 0.0..values_range.end as f32)
        .unwrap();

    chart
        .configure_mesh()
        .y_labels(umas.len())
        .light_line_style(WHITE)
        .draw()
        .unwrap();

    for (label, (values, style, offset)) in &series {
        chart
            .draw_series(values.iter().map(|x| {
                Boxplot::new_vertical(SegmentValue::CenterOf(&x.0), x.1)
                    .width(30)
                    .whisker_width(1.5)
                    .style(style.filled())
                    .offset(*offset)
            }))
            .unwrap()
            .label(label);
    }
}

fn write_box_plot(name: &str) -> SVGBackend {
    // should base this on the amount of umas
    SVGBackend::new(name, (1024, 768))
}

pub struct UmaData {
    pub name: String,
    pub scores: Vec<u32>,
}

impl UmaData {
    pub fn mean_score(&self) -> u32 {
        self.scores.iter().sum::<u32>() / self.scores.iter().count() as u32
    }
}

enum BoxPlotType {
    Min,
    Mean,
    Max,
}

impl BoxPlotType {
    fn to_comparer(&self) -> fn(&UmaData) -> u32 {
        match self {
            Min => |x| *x.scores.iter().min().unwrap(),
            Mean => UmaData::mean_score,
            Max => |x| *x.scores.iter().max().unwrap(),
        }
    }
}

impl Display for BoxPlotType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Min => "Minimum",
            Mean => "Mean",
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
