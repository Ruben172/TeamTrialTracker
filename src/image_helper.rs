use std::collections::HashMap;
use std::path::Path;

use image::{DynamicImage, GenericImageView, ImageReader};
use imageproc::distance_transform::Norm;
use imageproc::edges::canny;
use imageproc::morphology::dilate;
use imageproc::region_labelling::{Connectivity, connected_components};

#[derive(Clone, Debug)]
struct Rectangle {
    left: u32,
    top: u32,
    width: u32,
    height: u32,
}

pub fn decode_image(path: &Path) -> DynamicImage {
    ImageReader::open(path)
        .expect("Failed to open image")
        .with_guessed_format()
        .unwrap()
        .decode()
        .expect("Failed to decode image")
}

pub fn auto_crop_image(img: DynamicImage) -> DynamicImage {
    let rectangle = get_standard_crop_area(img.dimensions());
    // Standard crop region found
    if rectangle.width != img.dimensions().0 {
        return crop_image(img, &rectangle);
    }
    // No matching resolution
    println!(
        "Unusual resolution detected ({}x{}). Trying to find region with scores...",
        img.dimensions().0,
        img.dimensions().1
    );
    let found_rectangle = find_crop_area(&img);
    if let Some(rectangle) = found_rectangle {
        crop_image(img, &rectangle)
    } else {
        println!(
            "Couldn't find region, continuing with uncropped image ({}x{})",
            img.dimensions().0,
            img.dimensions().1
        );
        img
    }
}

fn crop_image(mut img: DynamicImage, rectangle: &Rectangle) -> DynamicImage {
    img.crop(
        rectangle.left,
        rectangle.top,
        rectangle.width,
        rectangle.height,
    )
}

fn get_standard_crop_area(dimensions: (u32, u32)) -> Rectangle {
    match dimensions {
        (1920, 1080) => Rectangle {
            left: 375,
            top: 90,
            width: 450,
            height: 852,
        }, // 1080p
        (2560, 1440) => Rectangle {
            left: 510,
            top: 125,
            width: 585,
            height: 1130,
        }, // 2K
        (3840, 2160) => Rectangle {
            left: 760,
            top: 190,
            width: 890,
            height: 1693,
        }, // 4K
        (1680, 1050) => Rectangle {
            left: 330,
            top: 135,
            width: 392,
            height: 742,
        }, // 1680x1050
        (1170, 2532) => Rectangle {
            left: 250,
            top: 406,
            width: 860,
            height: 1635,
        }, // iPhone 12
        (1080, 2340) => Rectangle {
            left: 230,
            top: 380,
            width: 795,
            height: 1503,
        }, // Samsung Galaxy s24
        (w, h) => Rectangle {
            left: 0,
            top: 0,
            width: w,
            height: h,
        },
    }
}

fn find_crop_area(img: &DynamicImage) -> Option<Rectangle> {
    let low_threshold = 50.0;
    let high_threshold = 60.0;
    let min_aspect_ratio = 0.55;
    let max_aspect_ratio = 0.64;

    let gray = img.to_luma8();
    let edges = canny(&gray, low_threshold, high_threshold);
    let edges = dilate(&edges, Norm::L1, 2);
    let components = connected_components(&edges, Connectivity::Eight, image::Luma([0]));

    let mut bounds = HashMap::<u32, Rectangle>::new();

    // Save the components in a hashmap storing them as rectangles
    for (x, y, pixel) in components.enumerate_pixels() {
        let label = pixel[0];

        // Background
        if label == 0 {
            continue;
        }

        bounds
            .entry(label)
            .and_modify(|b| {
                b.left = b.left.min(x);
                b.top = b.top.min(y);
                b.width = b.width.max(x - b.left);
                b.height = b.height.max(y - b.top);
            })
            .or_insert(Rectangle {
                left: x,
                top: y,
                width: 0,
                height: 0,
            });
    }

    // Find a region that matches the expected aspect ratio
    for rect in bounds.values() {
        let aspect_ratio = f64::from(rect.width) / f64::from(rect.height);

        if rect.width > 200
            && rect.height > 300
            && (min_aspect_ratio..=max_aspect_ratio).contains(&aspect_ratio)
        {
            return Some(shrink_crop_area(rect));
        }
    }

    // No region found
    None
}

#[allow(clippy::cast_possible_truncation)]
fn shrink_crop_area(r: &Rectangle) -> Rectangle {
    Rectangle {
        left: r.left + (f64::from(r.width) * 0.215) as u32,
        top: r.top,
        width: (f64::from(r.width) * 0.741) as u32,
        height: r.height,
    }
}
