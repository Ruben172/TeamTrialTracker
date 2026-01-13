use crate::uma::uma_fill_color;
use plotly::{
    common::{
        Anchor::{Left, Top},
        Line,
    },
    layout::Annotation,
    BoxPlot, Layout, Plot,
};
use plotly_static::{ImageFormat, StaticExporter, StaticExporterBuilder};
use std::{
    collections::HashMap,
    env,
    fmt::Display,
    path::{Path, PathBuf},
};
use BoxPlotType::*;

const GECKO_PATH: &str = "./geckodriver";

pub fn create_plots(umadata: &mut Vec<UmaData>) -> Vec<PlotWrapper> {
    let min = make_box_plot(umadata, Min);
    let mean = make_box_plot(umadata, Mean);
    let max = make_box_plot(umadata, Max);

    vec![min, mean, max]
}

pub fn render_plots(plots: Vec<PlotWrapper>) {
    println!(
        "Rendering box plots... do not close the application (or you will have to manually kill geckodriver)"
    );
    let webdriver_path = PathBuf::from(GECKO_PATH);
    unsafe {
        env::set_var("WEBDRIVER_PATH", webdriver_path);
    }

    let mut exporter = StaticExporterBuilder::default()
        .build()
        .expect("Failed to build StaticExporter");

    for plot in plots {
        render_plot(plot, &mut exporter);
    }

    exporter.close()
}

fn make_box_plot(umadata: &mut Vec<UmaData>, box_plot_type: BoxPlotType) -> PlotWrapper {
    let comparer = box_plot_type.to_comparer();
    umadata.sort_by_key(&comparer);

    let shown_score: u32 = umadata.iter().map(comparer).sum();
    let label = Annotation::new()
        .text(format!(
            r#"{} Team Trial score:<br>{}"#,
            box_plot_type, shown_score
        ))
        .x_ref("paper")
        .y_ref("paper")
        .x(0)
        .y(1.16)
        .x_anchor(Left)
        .y_anchor(Top)
        .show_arrow(false)
        .background_color("#aaa3");
    let layout = Layout::new()
        .title("Team Trials scores")
        .show_legend(false)
        .annotations(vec![label]);

    let mut plot = Plot::new();
    plot.set_layout(layout);
    for uma in umadata {
        let name = uma.name.clone();
        let color = uma_fill_color(&name);
        let trace = BoxPlot::new(uma.scores.clone())
            .name(&name)
            .fill_color(color.clone() + "b3") // 0.7 opacity
            .line(Line::new().color(darken_hex(&color)).width(1.5));
        plot.add_trace(trace);
    }

    PlotWrapper {
        box_plot_type,
        plot,
    }
}

fn render_plot(plot: PlotWrapper, exporter: &mut StaticExporter) {
    exporter
        .write_fig(
            Path::new(format!("./output/{}.png", plot.box_plot_type.to_file_name()).as_str()),
            &serde_json::from_str(&plot.plot.to_json()).expect("Failed to serialise boxplot"),
            ImageFormat::PNG,
            800,
            600,
            1.0,
        )
        .expect("Failed to export plot");
}

pub struct UmaData {
    pub name: String,
    pub scores: Vec<u32>,
}

impl UmaData {
    pub fn mean_score(&self) -> u32 {
        self.scores.iter().sum::<u32>() / self.scores.len() as u32
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

pub struct PlotWrapper {
    box_plot_type: BoxPlotType,
    plot: Plot,
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

    fn to_file_name(&self) -> &str {
        match self {
            Min => "min",
            Mean => "mean",
            Max => "max",
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
