use std::env;
use std::fs::File;
use std::io::BufReader;
use exif::{In, Tag, Value};
use reqwest::header::USER_AGENT;
use serde::Deserialize;
use colored::*;

#[derive(Debug, Deserialize)]
struct NominatimResponse {
    display_name: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("{}", "Error: No photo path provided.".red().bold());
        println!("Usage: {} <photo_path>", args[0].cyan());
        return Ok(());
    }

    let file_path = &args[1];
    println!("🔍 {} {}", "Analyzing Photo:".yellow(), file_path.blue().bold());

    let file = File::open(file_path).map_err(|e| {
        eprintln!("{} {}", "❌ Failed to open file:".red().bold(), e);
        e
    })?;
    
    let mut reader = BufReader::new(file);
    let exifreader = exif::Reader::new();
    
    let exif = match exifreader.read_from_container(&mut reader) {
        Ok(exif) => exif,
        Err(e) => {
            println!("{} {}", "❌ Error reading EXIF data:".red().bold(), e);
            println!("{}", "Note: This image might not have metadata (many social media apps strip it).".white().italic());
            return Ok(());
        }
    };

    println!("\n--- {} ---", "EXTRACTED GPS DATA".bold().cyan());

    let lat_data = get_gps_data(&exif, Tag::GPSLatitude, Tag::GPSLatitudeRef);
    let lon_data = get_gps_data(&exif, Tag::GPSLongitude, Tag::GPSLongitudeRef);
    let altitude = get_altitude(&exif);
    let direction = get_tag_value(&exif, Tag::GPSImgDirection);
    let date = get_tag_value(&exif, Tag::GPSDateStamp);
    let time = get_tag_value(&exif, Tag::GPSTimeStamp);

    match (lat_data, lon_data) {
        (Some((lat_dms, lat_dec)), Some((lon_dms, lon_dec))) => {
            println!("📌 {}:  {}", "Latitude".green(), lat_dms.cyan());
            println!("📌 {}: {}", "Longitude".green(), lon_dms.cyan());
            println!("🔢 {}:   {:.6}, {:.6}", "Decimal".green(), lat_dec, lon_dec);

            if let Some(alt) = altitude {
                println!("⛰️ {}:  {}", "Altitude".green(), alt.cyan());
            }
            if let Some(dir) = direction {
                println!("🧭 {}: {}", "Direction".green(), dir.cyan());
            }
            if let (Some(d), Some(t)) = (date, time) {
                println!("📅 {}:      {} {}", "GPS Time".green(), d.cyan(), t.cyan());
            }

            lookup_address(lat_dec, lon_dec).await?;
        }
        _ => {
            println!("{}", "⚠️ No GPS coordinates found in image metadata.".yellow().bold());
            println!("{}", "Tip: Use photos taken directly from a camera app (iPhone/Android).".white().italic());
        }
    }

    Ok(())
}

fn get_gps_data(exif: &exif::Exif, tag: Tag, ref_tag: Tag) -> Option<(String, f64)> {
    let field = exif.get_field(tag, In::PRIMARY)?;
    let ref_field = exif.get_field(ref_tag, In::PRIMARY)?;
    
    let dms_str = format!("{} {}", field.display_value(), ref_field.display_value());
    
    if let Value::Rational(rationals) = &field.value {
        if rationals.len() >= 3 {
            let degrees = rationals[0].to_f64();
            let minutes = rationals[1].to_f64();
            let seconds = rationals[2].to_f64();

            let mut decimal = degrees + (minutes / 60.0) + (seconds / 3600.0);
            let ref_val = ref_field.display_value().to_string();
            if ref_val.contains('S') || ref_val.contains('W') {
                decimal = -decimal;
            }
            return Some((dms_str, decimal));
        }
    }
    None
}

fn get_altitude(exif: &exif::Exif) -> Option<String> {
    let alt = exif.get_field(Tag::GPSAltitude, In::PRIMARY)?;
    let ref_val = exif.get_field(Tag::GPSAltitudeRef, In::PRIMARY)
        .map(|f| f.display_value().to_string())
        .unwrap_or_else(|| "Above Sea Level".to_string());
    
    Some(format!("{} ({})", alt.display_value(), ref_val))
}

fn get_tag_value(exif: &exif::Exif, tag: Tag) -> Option<String> {
    exif.get_field(tag, In::PRIMARY).map(|f| f.display_value().to_string())
}

async fn lookup_address(lat: f64, lon: f64) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n{} {}", "🌐".cyan(), "Connecting to OpenStreetMap for location details...".dimmed());
    
    let url = format!(
        "https://nominatim.openstreetmap.org/reverse?format=json&lat={}&lon={}&zoom=18&addressdetails=1",
        lat, lon
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let res = client
        .get(url)
        .header(USER_AGENT, "GeoPic-Official-Linux-Tool/1.0 (Contact: efe-rust-dev)")
        .send()
        .await;

    match res {
        Ok(response) => {
            if response.status().is_success() {
                let json: NominatimResponse = response.json().await?;
                println!("✅ {}","Exact Address:".green().bold());
                println!("{}", json.display_name.white().bold());
                
                println!("\n🔗 {}","System Shortcut:".blue().bold());
                println!("   Google Maps: https://www.google.com/maps?q={},{}", lat, lon);
                println!("   OSM:         https://www.openstreetmap.org/?mlat={}&mlon={}", lat, lon);
            } else if response.status() == 429 {
                println!("{}", "❌ Rate Limit Hit: Too many requests. Please wait a minute.".red().bold());
            } else {
                println!("{} {}", "❌ API Error:".red().bold(), response.status());
            }
        }
        Err(e) => {
            if e.is_timeout() {
                println!("{}", "❌ Connection Timeout: The server took too long to respond.".red().bold());
            } else {
                println!("{} {}", "❌ Network Error:".red().bold(), e);
            }
        }
    }

    Ok(())
}
