use std::{
    io::{BufWriter, Cursor},
    path::Path,
};

use image::{DynamicImage, GenericImageView, ImageReader};

pub fn dynamic_image_to_bytes(img: &DynamicImage) -> Vec<u8> {
    let mut buf = Vec::new();
    img.write_to(
        BufWriter::new(&mut Cursor::new(&mut buf)),
        image::ImageFormat::Tiff,
    )
    .expect("Failed to encode image");
    buf
}

pub fn decode_image(path: &Path) -> DynamicImage {
    ImageReader::open(path)
        .expect("Failed to open image")
        .with_guessed_format()
        .unwrap()
        .decode()
        .expect("Failed to decode image")
}

pub fn crop_image(mut img: DynamicImage) -> DynamicImage {
    let rectangle = get_rectangle(img.dimensions());
    img.crop(
        rectangle.left,
        rectangle.top,
        rectangle.width,
        rectangle.height,
    )
}

struct Rectangle {
    left: u32,
    top: u32,
    width: u32,
    height: u32,
}

fn get_rectangle(dimensions: (u32, u32)) -> Rectangle {
    match dimensions {
        (1920, 1080) => Rectangle {
            left: 375,
            top: 90,
            width: 450,
            height: 852,
        }, // 1080p
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
        (w, h) => {
            println!("Continuing with uncropped image ({w}x{h})");
            Rectangle {
                left: 0,
                top: 0,
                width: w,
                height: h,
            }
        }
    }
}
