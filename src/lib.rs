mod utils;

use quircs::CodeIter;
use types::ScanResult;
use wasm_bindgen::{prelude::*, Clamped};
use web_sys::ImageData;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

mod types;

#[wasm_bindgen]
pub fn read_qrcodes_from_image_data(
    img: ImageData,
    process_image_to_find_more: bool,
) -> ScanResult {
    let mut decoder = quircs::Quirc::default(); // allocates stuff, so better move into class to be reused
    let image_buffer = image::RgbaImage::from_raw(img.width(), img.height(), img.data().to_vec())
        .expect("read image");
    let image = image::DynamicImage::from(image_buffer);
    let img_gray = image.into_luma8();

    let mut codes: Vec<_> = decoder
        .identify(
            img_gray.width() as usize,
            img_gray.height() as usize,
            &img_gray,
        )
        .collect();

    if process_image_to_find_more {
        // try with adaptive_threshold small block_radius
        let mut img_gray_threshold = imageproc::contrast::adaptive_threshold(&img_gray, 4);
        if codes.is_empty() || codes.first().is_some_and(|qr| qr.is_err()) {
            codes = decoder
                .identify(
                    img_gray.width() as usize,
                    img_gray.height() as usize,
                    &img_gray_threshold,
                )
                .collect();
        }
        // try with adaptive_threshold normal block_radius
        img_gray_threshold = imageproc::contrast::adaptive_threshold(&img_gray, 10);
        if codes.is_empty() || codes.first().is_some_and(|qr| qr.is_err()) {
            codes = decoder
                .identify(
                    img_gray.width() as usize,
                    img_gray.height() as usize,
                    &img_gray_threshold,
                )
                .collect();
        }
        // try with an inverted version of adaptive_threshold
        if codes.is_empty() || codes.first().is_some_and(|qr| qr.is_err()) {
            image::imageops::invert(&mut img_gray_threshold);
            codes = decoder
                .identify(
                    img_gray.width() as usize,
                    img_gray.height() as usize,
                    &img_gray_threshold,
                )
                .collect();
        }
    }

    let mut result = Vec::new();

    codes.iter().for_each(|code| {
        if let Ok(code) = code {
            result.push(types::QRCode {
                corners: code.corners.map(|p| p.into()),
                data: match code.decode() {
                    Ok(data) => data.into(),
                    Err(err) => types::QRCodeDecodingResult::Error(err.to_string()),
                },
            })
        }
    });
    ScanResult(result)
}

/// Only for debugging and improving understanding
#[wasm_bindgen]
pub fn luma_image_data(
    img: ImageData,
    threshold: bool,
    inverted: bool,
) -> std::result::Result<web_sys::ImageData, wasm_bindgen::JsValue> {
    let image_buffer = image::RgbaImage::from_raw(img.width(), img.height(), img.data().to_vec())
        .expect("read image");
    let image = image::DynamicImage::from(image_buffer);
    let mut img_gray = image.into_luma8();
    if threshold {
        img_gray = imageproc::contrast::adaptive_threshold(&img_gray, 10);
    }
    if inverted {
        image::imageops::invert(&mut img_gray);
    }
    let t = img_gray.to_vec();
    let mut img2: Vec<u8> = Vec::with_capacity(t.len() * 4);
    for p in t {
        img2.extend([p, p, p, 255]);
    }

    ImageData::new_with_u8_clamped_array_and_sh(Clamped(&img2[..]), img.width(), img.height())
}
