mod metadata;
mod geocoding;

use std::path::Path;
use nom_exif::{MediaParser, MediaSource, ExifIter};
use colored::*;
use clap::Parser;
use crate::metadata::{PhotoMetadata, extract_metadata};
use crate::geocoding::GeocodingClient;

#[derive(Parser, Debug)]
#[command(author, version, about = "GeoPic: Fotoğraf Konum ve Veri Analiz Aracı", long_about = None)]
struct Args {
    /// Analiz edilecek fotoğraf yolu
    path: String,

    /// İngilizce çıktı verir
    #[arg(short, long)]
    en: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let is_tr = !args.en;

    print_banner(is_tr);

    let file_path = &args.path;
    if !Path::new(file_path).exists() {
        let err_msg = if is_tr { "❌ Dosya bulunamadı:" } else { "❌ File not found:" };
        println!("{} {}", err_msg.red().bold(), file_path);
        return Ok(());
    }

    let analyze_msg = if is_tr { "🔍 Fotoğraf Analiz Ediliyor:" } else { "🔍 Analyzing Photo:" };
    println!("{} {}", analyze_msg.yellow(), file_path.blue().bold());

    // Initialize Parser
    let mut parser = MediaParser::new();
    let ms = MediaSource::file_path(file_path).map_err(|e| {
        let open_err = if is_tr { "❌ Dosya açılamadı:" } else { "❌ Failed to open file:" };
        eprintln!("{} {}", open_err.red().bold(), e);
        e
    })?;

    let iter: ExifIter = match parser.parse(ms) {
        Ok(iter) => iter,
        Err(e) => {
            let parse_err = if is_tr { "❌ Meta veriler okunamadı:" } else { "❌ Error parsing metadata:" };
            let note_err = if is_tr { 
                "Not: Bu görselde EXIF başlığı olmayabilir veya format desteklenmiyor." 
            } else { 
                "Note: This image might not have EXIF headers or the format is unsupported." 
            };
            println!("{} {}", parse_err.red().bold(), e);
            println!("{}", note_err.white().italic());
            return Ok(());
        }
    };

    let meta = extract_metadata(iter);

    println!("\n--- {} ---", if is_tr { "FOTOĞRAF DETAYLARI" } else { "PHOTO DETAILS" }.bold().cyan());

    display_metadata(&meta, is_tr);

    if let (Some(lat), Some(lon)) = (meta.latitude, meta.longitude) {
        let client = GeocodingClient::new(is_tr);
        match client.lookup(lat, lon).await {
            Ok(res) => {
                let addr_title = if is_tr { "✅ Kesin Adres:" } else { "✅ Exact Address:" };
                println!("{} {}", addr_title.green().bold(), res.display_name.white().bold());
                
                let map_title = if is_tr { "🔗 Haritada Aç:" } else { "🔗 Open Maps:" };
                println!("\n{} ", map_title.blue().bold());
                println!("   Google Maps: https://www.google.com/maps?q={},{}", lat, lon);
                println!("   OSM:         https://www.openstreetmap.org/?mlat={}&mlon={}", lat, lon);
            }
            Err(e) => println!("{}", e.to_string().red().bold()),
        }
    } else {
        let no_gps = if is_tr { 
            "⚠️ Fotoğrafta GPS koordinatı bulunamadı." 
        } else { 
            "⚠️ No GPS coordinates found in metadata." 
        };
        let tip_txt = if is_tr {
            "İpucu: Sosyal medya (WhatsApp/Instagram) üzerinden gelen fotoğraflar genellikle bu veriyi siler."
        } else {
            "Note: Social media (WhatsApp/Instagram) usually strips this data for privacy."
        };
        println!("{}", no_gps.yellow().bold());
        println!("{}", tip_txt.white());
    }

    Ok(())
}

fn display_metadata(meta: &PhotoMetadata, is_tr: bool) {
    let labels = if is_tr {
        vec!["Enlem", "Boylam", "Yükseklik", "Zaman", "Cihaz", "Model", "Lens", "Pozlama", "Diyafram", "ISO"]
    } else {
        vec!["Latitude", "Longitude", "Altitude", "Timestamp", "Make", "Model", "Lens", "Exposure", "F-Number", "ISO"]
    };

    if let Some(lat) = meta.latitude { println!("📍 {}:  {:.6}", labels[0].green(), lat); }
    if let Some(lon) = meta.longitude { println!("📍 {}: {:.6}", labels[1].green(), lon); }
    if let Some(alt) = meta.altitude { println!("⛰️ {}:  {:.2}m", labels[2].green(), alt); }
    if let Some(ts) = &meta.timestamp { println!("📅 {}:      {}", labels[3].green(), ts.cyan()); }
    
    println!("\n--- {} ---", if is_tr { "TEKNİK BİLGİLER" } else { "TECHNICAL INFO" }.bold().dimmed());
    if let Some(v) = &meta.make { println!("📷 {}:   {}", labels[4].yellow(), v); }
    if let Some(v) = &meta.model { println!("📱 {}:   {}", labels[5].yellow(), v); }
    if let Some(v) = &meta.lens { println!("🔍 {}:   {}", labels[6].yellow(), v); }
    if let Some(v) = &meta.exposure { println!("⏱️ {}:{}", labels[7].yellow(), v); }
    if let Some(v) = &meta.f_number { println!("🔦 {}:{}", labels[8].yellow(), v); }
    if let Some(v) = &meta.iso { println!("🎞️ {}:     {}", labels[9].yellow(), v); }
}

fn print_banner(is_tr: bool) {
    println!("{}", "===============================================".dimmed());
    println!("{}", "   GeoPic - High Precision Location Finder     ".bold().green());
    println!("{}", "===============================================".dimmed());
    
    if is_tr {
        println!("{}", "⚖️  GİZLİLİK VE ETİK UYARISI:".bold().yellow());
        println!("{}", "- Bu aracı sadece sahibi olduğunuz veya izniniz olan fotoğraflar için kullanın.");
        println!("{}", "- Takip, taciz veya ifşa (doxing) amaçlı kullanmayın.");
        println!("{}", "- Yerel veri gizliliği yasalarına saygı gösterin.");
    } else {
        println!("{}", "⚖️  PRIVACY & ETHICS NOTICE:".bold().yellow());
        println!("{}", "- Use this tool only on photos you own or have permission for.");
        println!("{}", "- Do not use for stalking, harassment, or doxing.");
        println!("{}", "- Respect local laws regarding geolocation data privacy.");
    }
    println!("{}\n", "===============================================".dimmed());
}
