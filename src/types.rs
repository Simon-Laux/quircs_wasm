use serde::Serialize;
use tsify::Tsify;
use wasm_bindgen::prelude::*;

#[derive(Tsify, Debug, Serialize)]
#[tsify(into_wasm_abi)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl From<quircs::Point> for Point {
    fn from(p: quircs::Point) -> Self {
        Self { x: p.x, y: p.y }
    }
}

#[derive(Tsify, Debug, Serialize)]
#[tsify(into_wasm_abi)]
pub enum QRCodeDecodingResult {
    Ok(Data),
    Error(String),
}

impl From<quircs::Data> for QRCodeDecodingResult {
    fn from(data: quircs::Data) -> Self {
        QRCodeDecodingResult::Ok(Data {
            version: data.version,
            ecc_level: match data.ecc_level {
                quircs::EccLevel::M => EccLevel::M,
                quircs::EccLevel::L => EccLevel::L,
                quircs::EccLevel::H => EccLevel::H,
                quircs::EccLevel::Q => EccLevel::Q,
            },
            mask: data.mask,
            data_type: data.data_type.map(|data| match data {
                quircs::DataType::Numeric => DataType::Numeric,
                quircs::DataType::Alpha => DataType::Alpha,
                quircs::DataType::Byte => DataType::Byte,
                quircs::DataType::Eci => DataType::Eci,
                quircs::DataType::Kanji => DataType::Kanji,
            }),
            payload: data.payload,
        })
    }
}

#[derive(Tsify, Debug, Serialize)]
#[tsify(into_wasm_abi)]
pub struct QRCode {
    pub corners: [Point; 4],
    pub data: QRCodeDecodingResult,
}

#[derive(Tsify, Debug, Serialize)]
#[tsify(into_wasm_abi)]
pub struct ScanResult(pub Vec<QRCode>);

/// QR-code ECC types.
#[derive(Tsify, Debug, Clone, Serialize)]
#[tsify(into_wasm_abi)]
pub enum EccLevel {
    M = 0,
    L = 1,
    H = 2,
    Q = 3,
}

/// QR-code data types.
#[derive(Tsify, Debug, Clone, Serialize)]
#[tsify(into_wasm_abi)]
#[repr(i32)]
pub enum DataType {
    Numeric = 1,
    Alpha = 2,
    Byte = 4,
    Eci = 7,
    Kanji = 8,
}

/// This structure holds the decoded QR-code data
#[derive(Tsify, Debug, Clone, Serialize)]
#[tsify(into_wasm_abi)]
pub struct Data {
    ///  Various parameters of the QR-code. These can mostly be  ignored
    /// if you only care about the data.
    pub version: usize,
    pub ecc_level: EccLevel,
    pub mask: i32,
    /// This field is the highest-valued data type found in the QR code.
    pub data_type: Option<DataType>,
    /// Data payload. For the Kanji datatype, payload is encoded as Shift-JIS.
    /// For all other datatypes, payload is ASCII text.
    pub payload: Vec<u8>,
}
