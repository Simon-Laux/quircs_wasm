mod utils;

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
pub fn read_qrcodes_from_image_data(img: ImageData) -> ScanResult {
    let image_buffer = image::RgbaImage::from_raw(img.width(), img.height(), img.data().to_vec())
        .expect("read image");
    let image = image::DynamicImage::from(image_buffer);
    let img_gray = image.into_luma8();
    let img_gray = imageproc::contrast::adaptive_threshold(&img_gray, 10);
    let mut decoder = quircs::Quirc::default();
    let codes = decoder.identify(
        img_gray.width() as usize,
        img_gray.height() as usize,
        &img_gray,
    );

    let mut result = Vec::new();

    codes.for_each(|code| {
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

#[wasm_bindgen]
pub fn luma_image_data(
    img: ImageData,
) -> std::result::Result<web_sys::ImageData, wasm_bindgen::JsValue> {
    let image_buffer = image::RgbaImage::from_raw(img.width(), img.height(), img.data().to_vec())
        .expect("read image");
    let image = image::DynamicImage::from(image_buffer);
    let img_gray = image.into_luma8();
    let img_gray = imageproc::contrast::adaptive_threshold(&img_gray, 10);

    let t = img_gray.to_vec();
    let mut img2: Vec<u8> = Vec::with_capacity(t.len() * 4);
    for p in t {
        img2.extend([p, p, p, 255]);
    }

    ImageData::new_with_u8_clamped_array_and_sh(Clamped(&img2[..]), img.width(), img.height())
}
