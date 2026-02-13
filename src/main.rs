use std::env;
use std::path::Path;
use nom_exif::{MediaParser, MediaSource, ExifIter};
use reqwest::header::USER_AGENT;
use serde::Deserialize;
use colored::*;

#[derive(Debug, Deserialize)]
struct NominatimResponse {
    display_name: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_banner();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("{}", "Error: No photo path provided.".red().bold());
        println!("Usage: {} <photo_path>", args[0].cyan());
        return Ok(());
    }

    let file_path = &args[1];
    if !Path::new(file_path).exists() {
        println!("{} {}", "❌ File not found:".red().bold(), file_path);
        return Ok(());
    }

    println!("🔍 {} {}", "Analyzing Photo:".yellow(), file_path.blue().bold());

    // Initialize Parser
    let mut parser = MediaParser::new();
    let ms = MediaSource::file_path(file_path).map_err(|e| {
        eprintln!("{} {}", "❌ Failed to open file:".red().bold(), e);
        e
    })?;

    let iter: ExifIter = match parser.parse(ms) {
        Ok(iter) => iter,
        Err(e) => {
            println!("{} {}", "❌ Error parsing metadata:".red().bold(), e);
            println!("{}", "Note: This image might not have EXIF headers or the format is unsupported.".white().italic());
            return Ok(());
        }
    };

    println!("\n--- {} ---", "EXTRACTED GPS DATA".bold().cyan());

    // nom-exif 2.0+ provides a helper for GPS info
    if let Some(gps_info) = iter.parse_gps_info().ok().flatten() {
        let lat = convert_latlng(&gps_info.latitude, gps_info.latitude_ref);
        let lon = convert_latlng(&gps_info.longitude, gps_info.longitude_ref);

        println!("📌 {}:  {:.6}", "Latitude".green(), lat);
        println!("📌 {}: {:.6}", "Longitude".green(), lon);

        let a = gps_info.altitude;
        let alt_val = a.0 as f64 / a.1 as f64;
        println!("⛰️ {}:  {:.2}m", "Altitude".green(), alt_val);

        lookup_address(lat, lon).await?;
    } else {
        println!("{}", "⚠️ No GPS coordinates found in image metadata.".yellow().bold());
        println!("{}", "Note: Social media (WhatsApp/Instagram) and 'Save for Web' options strip this data.".white());
    }

    Ok(())
}

fn convert_latlng(ll: &nom_exif::LatLng, ref_char: char) -> f64 {
    let d = ll.0.0 as f64 / ll.0.1 as f64;
    let m = ll.1.0 as f64 / ll.1.1 as f64;
    let s = ll.2.0 as f64 / ll.2.1 as f64;

    let mut res = d + (m / 60.0) + (s / 3600.0);
    if ref_char == 'S' || ref_char == 'W' {
        res = -res;
    }
    res
}

fn print_banner() {
    println!("{}", "===============================================".dimmed());
    println!("{}", "   GeoPic - High Precision Location Finder     ".bold().green());
    println!("{}", "===============================================".dimmed());
    println!("{}", "⚖️  PRIVACY & ETHICS NOTICE:".bold().yellow());
    println!("{}", "- Use this tool only on photos you own or have permission for.");
    println!("{}", "- Do not use for stalking, harassment, or doxing.");
    println!("{}", "- Respect local laws regarding geolocation data privacy.");
    println!("{}\n", "===============================================".dimmed());
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
        .header(USER_AGENT, "GeoPic-Official-Linux-Tool/2.0 (Contact: efe-rust-dev)")
        .send()
        .await;

    match res {
        Ok(response) => {
            if response.status().is_success() {
                let json: NominatimResponse = response.json().await?;
                println!("✅ {}","Exact Address:".green().bold());
                println!("{}", json.display_name.white().bold());
                
                println!("\n🔗 {}","Open Maps:".blue().bold());
                println!("   Google Maps: https://www.google.com/maps?q={},{}", lat, lon);
                println!("   OSM:         https://www.openstreetmap.org/?mlat={}&mlon={}", lat, lon);
            } else if response.status() == 429 {
                println!("{}", "❌ Rate Limit Hit: Nominatim API is busy. Please try again in 1 minute.".red().bold());
            } else {
                println!("{} {}", "❌ API Error:".red().bold(), response.status());
            }
        }
        Err(e) => {
            if e.is_timeout() {
                println!("{}", "❌ Connection Timeout: API took too long to respond.".red().bold());
            } else {
                println!("{} {}", "❌ Network Error:".red().bold(), e);
            }
        }
    }

    Ok(())
}
