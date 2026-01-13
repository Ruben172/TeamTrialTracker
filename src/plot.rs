use crate::uma::uma_fill_color;
use std::{
    collections::HashMap,
    env,
    fmt::Display,
    path::{Path, PathBuf},
};
use charming::{Chart, ImageRenderer};
use charming::component::{Axis, Legend, Title};
use charming::datatype::{DataSource, Dataset};
use charming::series::Boxplot;
use image::ImageFormat;
use BoxPlotType::*;

const GECKO_PATH: &str = "./geckodriver";

pub fn create_charts(umadata: &mut Vec<UmaData>) -> Vec<ChartWrapper> {
    let min = make_box_plot(umadata, Min);
    let mean = make_box_plot(umadata, Mean);
    let max = make_box_plot(umadata, Max);

    vec![min, mean, max]
}

pub fn render_charts(plots: Vec<ChartWrapper>) {
    println!(
        "Rendering box plots... do not close the application (or you will have to manually kill geckodriver)"
    );

    for plot in plots {
        render_chart(&plot);
    }
}

fn make_box_plot(umadata: &mut Vec<UmaData>, box_plot_type: BoxPlotType) -> ChartWrapper {
    let comparer = box_plot_type.to_comparer();
    umadata.sort_by_key(&comparer);

    let shown_score: u32 = umadata.iter().map(comparer).sum();

    umadata.iter_mut().for_each(|x| x.scores.sort());
    let data: Vec<Vec<i32>> = umadata.iter().map(|x| x.scores.clone().iter_mut().map(|x| *x as i32).collect::<Vec<i32>>()).collect();

    // let ds = Dataset::new().transform();
    let chart = Chart::new()
        .title(Title::new().text("Team Trial scores"))
        .x_axis(Axis::new()/*.data(umadata.iter().map(|x| x.name.clone()).collect())*/)
        .y_axis(Axis::new().min(15000).max(60000))
        .series(Boxplot::new().name("boxplot").data(data));
    // todo!();
    // let label = Annotation::new()
    //     .text(format!(
    //         r#"{} Team Trial score:<br>{}"#,
    //         box_plot_type, shown_score
    //     ))
    //     .x_ref("paper")
    //     .y_ref("paper")
    //     .x(0)
    //     .y(1.16)
    //     .x_anchor(Left)
    //     .y_anchor(Top)
    //     .show_arrow(false)
    //     .background_color("#aaa3");
    // let layout = Layout::new()
    //     .title("Team Trials scores")
    //     .show_legend(false)
    //     .annotations(vec![label]);
    //
    // let mut plot = Plot::new();
    // plot.set_layout(layout);
    // for uma in umadata {
    //     let name = uma.name.clone();
    //     let color = uma_fill_color(&name);
    //     let trace = BoxPlot::new(uma.scores.clone())
    //         .name(&name)
    //         .fill_color(color.clone() + "b3") // 0.7 opacity
    //         .line(Line::new().color(darken_hex(&color)).width(1.5));
    //     plot.add_trace(trace);
    // }

    ChartWrapper {
        box_plot_type,
        chart,
    }
}

fn render_chart(chart: &ChartWrapper) {
    let mut renderer = ImageRenderer::new(800, 600);
    renderer.render_format(ImageFormat::Png, &chart.chart).unwrap();
    renderer.save_format(ImageFormat::Png, &chart.chart, format!("./output/{}.png", chart.box_plot_type.to_file_name())).unwrap()
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

pub struct ChartWrapper {
    box_plot_type: BoxPlotType,
    chart: Chart,
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