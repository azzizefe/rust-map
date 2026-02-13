use nom_exif::{ExifIter, LatLng};
use colored::*;

#[derive(Debug, Default)]
pub struct PhotoMetadata {
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub altitude: Option<f64>,
    pub timestamp: Option<String>,
    pub make: Option<String>,
    pub model: Option<String>,
    pub lens: Option<String>,
    pub exposure: Option<String>,
    pub f_number: Option<String>,
    pub iso: Option<String>,
}

pub fn extract_metadata(iter: ExifIter) -> PhotoMetadata {
    let mut meta = PhotoMetadata::default();
    
    // Parse GPS
    if let Some(gps) = iter.parse_gps_info().ok().flatten() {
        meta.latitude = Some(convert_latlng(&gps.latitude, gps.latitude_ref));
        meta.longitude = Some(convert_latlng(&gps.longitude, gps.longitude_ref));
        meta.altitude = Some(gps.altitude.0 as f64 / gps.altitude.1 as f64);
    }

    // Parse hardware/technical info
    for entry in iter {
        let tag_name = format!("{:?}", entry.tag());
        let value = entry.get_value().map(|v| v.to_string()).unwrap_or_default();

        match tag_name.as_str() {
            "Some(Make)" | "Make" => meta.make = Some(value),
            "Some(Model)" | "Model" => meta.model = Some(value),
            "Some(LensModel)" | "LensModel" | "Some(LensInfo)" | "LensInfo" => meta.lens = Some(value),
            "Some(ExposureTime)" | "ExposureTime" => meta.exposure = Some(value),
            "Some(FNumber)" | "FNumber" => meta.f_number = Some(value),
            "Some(ISOSpeedRatings)" | "ISOSpeedRatings" => meta.iso = Some(value),
            "Some(DateTimeOriginal)" | "DateTimeOriginal" | "Some(CreateDate)" | "CreateDate" => {
                if meta.timestamp.is_none() {
                    meta.timestamp = Some(value);
                }
            }
            _ => {}
        }
    }

    meta
}

fn convert_latlng(ll: &LatLng, ref_char: char) -> f64 {
    let d = ll.0.0 as f64 / ll.0.1 as f64;
    let m = ll.1.0 as f64 / ll.1.1 as f64;
    let s = ll.2.0 as f64 / ll.2.1 as f64;

    let mut res = d + (m / 60.0) + (s / 3600.0);
    if ref_char == 'S' || ref_char == 'W' {
        res = -res;
    }
    res
}
