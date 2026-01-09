#[warn(clippy::pedantic)]
use plotly::Layout;
use plotly::common::Line;
use plotly::{BoxPlot, Plot};
use plotly_static::{ImageFormat, StaticExporter, StaticExporterBuilder};
use regex::Regex;
use rusty_tesseract::Args;
use rusty_tesseract::Image;
use rusty_tesseract::image::{GenericImageView, ImageReader};
use std::collections::HashMap;
use std::convert::Into;
use std::{env, fs};
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
    // capture name, then capture score (comma included), has (?:.* )? in the middle in case MVP gets OCRed as bogus text
    let score_regex = Regex::new(r"^([A-Za-z.']+(?: [A-Za-z.']+)*) (?:.* )?(\d{1,3},\d{1,3})")
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
            (1920, 1080) => dynamic_image.clone().crop(375, 90, 450, 852), // 1080p
            (3840, 2160) => dynamic_image.clone().crop(760, 190, 890, 1693), // 4K
            (1680, 1050) => dynamic_image.clone().crop(330, 135, 392, 742), // 1680x1050
            (1170, 2532) => dynamic_image.clone().crop(250, 406, 860, 1635), // iPhone 12
            (1080, 2340) => dynamic_image.clone().crop(230, 380, 795, 1503), // Samsung Galaxy s24
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
                .filter(|x| horse_names.contains(&(x.to_string()))) // discard all words from name matches that do not exist in any uma's name.
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

    let serialised = serde_json::to_string_pretty(&scores).unwrap();
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

    let min = make_box_plot(&scores, |x| *x.scores.iter().min().unwrap());
    let mean = make_box_plot(&scores, UmaData::mean_score);
    let max = make_box_plot(&scores, |x| *x.scores.iter().max().unwrap());

    println!("Rendering box plots... do not close the application (or you will have to manually kill geckodriver)");
    let webdriver_path = PathBuf::from("./geckodriver");
    unsafe {
        env::set_var("WEBDRIVER_PATH", webdriver_path);
    }

    let mut exporter = StaticExporterBuilder::default()
        .build()
        .expect("Failed to build StaticExporter");
    write_box_plot("min", min, &mut exporter);
    write_box_plot("mean", mean, &mut exporter);
    write_box_plot("max", max, &mut exporter);

    exporter.close()
}

fn make_box_plot(scores: &HashMap<String, Vec<i32>>, comparer: fn(&UmaData) -> i32) -> Plot {
    let mut scores = scores
        .iter()
        .map(|(name, scores)| UmaData {
            name: name.clone(),
            scores: scores.clone(),
        })
        .collect::<Vec<UmaData>>();
    scores.sort_by(|x, y| comparer(x).cmp(&comparer(&y)));

    let layout = Layout::new().show_legend(false).title("Team Trials scores");

    let mut plot = Plot::new();
    plot.set_layout(layout);
    for uma in scores {
        let name = uma.name.clone();
        let color = uma_fill_color(&name);
        let trace = BoxPlot::new(uma.scores)
            .name(&name)
            .fill_color(color.clone() + "b3") // 0.7 opacity
            .line(Line::new().color(darken_hex(&color)).width(1.5));
        plot.add_trace(trace);
    }

    plot
}

fn uma_fill_color(name: &String) -> String {
    let name = name.as_str();
    match name {
        // "Admire Groove" => true,
        "Admire Vega" => "#608CCE",
        "Agnes Digital" => "#E29B98",
        "Agnes Tachyon" => "#DDCA89",
        "Air Groove" => "#836DAA",
        // "Air Messiah" => true,
        // "Air Shakur" => true,
        // "Almond Eye" => true,
        // "Aston Machan" => true,
        // "Bamboo Memory" => true,
        // "Believe" => true,
        // "Biko Pegasus" => true,
        "Biwa Hayahide" => "#D4B6B5",
        // "Blast Onepiece" => true,
        // "Bubble Gum Fellow" => true,
        // "Buena Vista" => true,
        // "Calstone Light O" => true,
        // "Cesario" => true,
        // "Cheval Grand" => true,
        // "Chrono Genesis" => true,
        // "Copano Rickey" => true,
        // "Curren Bouquetd'or" => true,
        "Curren Chan" => "#FFB7F0",
        // "Daiichi Ruby" => true,
        // "Daitaku Helios" => true,
        "Daiwa Scarlet" => "##4C65CB",
        // "Dantsu Flame" => true,
        // "Daring Heart" => true,
        // "Daring Tact" => true,
        // "Darley Arabian" => true,
        // "Dream Journey" => true,
        // "Duramente" => true,
        // "Durandal" => true,
        "Eishin Flash" => "#4F5167",
        "El Condor Pasa" => "#D72A13",
        // "Espoir City" => true,
        // "Fenomeno" => true,
        "Fine Motion" => "#43A26B",
        "Fuji Kiseki" => "#777B82",
        // "Furioso" => true,
        // "Fusaichi Pandora" => true,
        // "Gentildonna" => true,
        "Gold City" => "#F5C188",
        "Gold Ship" => "#E62D3F",
        // "Gran Alegria" => true,
        "Grass Wonder" => "#6279C1",
        // "Haiseiko" => true,
        // "Happy Meek" => true,
        "Haru Urara" => "#FEAEB0",
        "Hishi Akebono" => "#135FBE",
        "Hishi Amazon" => "#EFB371",
        // "Hishi Miracle" => true,
        // "Hokko Tarumae" => true,
        // "Ikuno Dictus" => true,
        // "Inari One" => true,
        // "Ines Fujin" => true,
        // "Jungle Pocket" => true,
        // "K.S. Miracle" => true,
        // "Katsuragi Ace" => true,
        "Kawakami Princess" => "#F1619F",
        "King Halo" => "#6279C1",
        "Kitasan Black" => "#D75B69",
        // "Loves Only You" => true,
        // "Lucky Lilac" => true,
        "Manhattan Cafe" => "#F2CD5B",
        "Maruzensky" => "#E04F27",
        // "Marvelous Sunday" => true,
        "Matikanefukukitaru" => "#ECC955",
        // "Matikanetannhauser" => true,
        "Mayano Top Gun" => "#FE9757",
        "Meisho Doto" => "#BE7055",
        // "Mejiro Ardan" => true,
        // "Mejiro Bright" => true,
        "Mejiro Dober" => "#6EBEC4",
        "Mejiro McQueen" => "#C0AEE0",
        // "Mejiro Palmer" => true,
        // "Mejiro Ramonu" => true,
        "Mejiro Ryan" => "#63BABC",
        "Mihono Bourbon" => "#FF5FFF",
        // "Mr. C.B." => true,
        // "Nakayama Festa" => true,
        "Narita Brian" => "#F3AE80",
        "Narita Taishin" => "#FCA0B9",
        // "Narita Top Road" => true,
        // "Neo Universe" => true,
        "Nice Nature" => "#B35B54",
        // "Nishino Flower" => true,
        // "No Reason" => true,
        // "North Flight" => true,
        "Oguri Cap" => "#D9D7DA",
        // "Orfevre" => true,
        "Rice Shower" => "#5A5182",
        // "Royce and Royce" => true,
        "Sakura Bakushin O" => "#FF78A7",
        // "Sakura Chitose O" => true,
        "Sakura Chiyono O" => "#FF9DBA",
        // "Sakura Laurel" => true,
        // "Satono Crown" => true,
        "Satono Diamond" => "#8DB087",
        // "Seeking the Pearl" => true,
        "Seiun Sky" => "#adcd91",
        // "Shinko Windy" => true,
        "Silence Suzuka" => "#DC8E68",
        // "Sirius Symboli" => true,
        "Smart Falcon" => "#FC67AD",
        // "Sounds of Earth" => true,
        "Special Week" => "#E877D2",
        // "Stay Gold" => true,
        // "Still in Love" => true,
        "Super Creek" => "#75BFDE",
        // "Sweep Tosho" => true,
        // "Symboli Kris S" => true,
        "Symboli Rudolf" => "#467F64",
        "T.M. Opera O" => "#FBB560",
        "Taiki Shuttle" => "#86CC48",
        "Tamamo Cross" => "#5C9EFD",
        // "Tanino Gimlet" => true,
        // "Tap Dance City" => true,
        "Tokai Teio" => "#587CE0",
        "Tosen Jordan" => "#5C71E5",
        // "Transcend" => true,
        // "Tsurumaru Tsuyoshi" => true,
        // "Twin Turbo" => true,
        "Vodka" => "#FEFF64",
        "Winning Ticket" => "#EF1D23",
        _ => "#000000",
    }.to_string()
}

// AI
fn hex_to_rgb(hex: &str) -> Option<(u8, u8, u8)> {
    let h = hex.trim_start_matches('#');
    if h.len() != 6 { return None; }
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

fn write_box_plot(name: &str, plot: Plot, exporter: &mut StaticExporter) -> () {
    exporter
        .write_fig(
            Path::new(format!("./output/{name}.png").as_str()),
            &serde_json::from_str(&plot.to_json()).expect("Failed to serialise boxplot"),
            ImageFormat::PNG,
            800,
            600,
            1.0,
        )
        .expect("Failed to export plot");
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
        psm: Some(6),
        oem: Some(1),
    }
}
