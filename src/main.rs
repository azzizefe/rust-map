mod metadata;
mod geocoding;
mod external;
mod export;

use std::path::Path;
use std::time::Duration;
use nom_exif::{MediaParser, MediaSource};
use colored::*;
use clap::Parser;
use walkdir::WalkDir;
use indicatif::{ProgressBar, ProgressStyle};
use crate::metadata::{PhotoMetadata, extract_metadata};
use crate::geocoding::GeocodingClient;
use crate::external::ExternalScanner;
use crate::export::{ExportRecord, save_to_json, save_to_csv};

#[derive(Parser, Debug)]
#[command(author, version, about = "GeoPic: Fotoğraf Konum ve Veri Analiz Aracı", long_about = None)]
struct Args {
    /// Analiz edilecek fotoğraf veya klasör yolu
    path: String,

    /// İngilizce çıktı verir
    #[arg(short, long)]
    en: bool,

    /// Harici araçlarla derin tarama yapar (ExifTool gerektirir)
    #[arg(short, long)]
    deep: bool,

    /// Belirtilen yolu bir klasör olarak tara ve içindeki tüm fotoğrafları işle
    #[arg(short, long)]
    batch: bool,

    /// Sonuçları belirtilen dosyaya kaydet (uzantıya göre .json veya .csv)
    #[arg(short, long)]
    output: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let is_tr = !args.en;

    print_banner(is_tr);

    let base_path = Path::new(&args.path);
    if !base_path.exists() {
        let err_msg = if is_tr { "❌ Yol bulunamadı:" } else { "❌ Path not found:" };
        println!("{} {}", err_msg.red().bold(), args.path);
        return Ok(());
    }

    let mut files_to_process = Vec::new();
    if args.batch && base_path.is_dir() {
        let scan_msg = if is_tr { "📁 Klasör taranıyor..." } else { "📁 Scanning directory..." };
        println!("{}", scan_msg.dimmed());
        for entry in WalkDir::new(base_path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let ext = entry.path().extension().and_then(|s| s.to_str()).unwrap_or_default().to_lowercase();
                if ["jpg", "jpeg", "heic", "heif", "tiff"].contains(&ext.as_str()) {
                    files_to_process.push(entry.path().to_string_lossy().to_string());
                }
            }
        }
    } else {
        files_to_process.push(args.path.clone());
    }

    let total = files_to_process.len();
    if total == 0 {
        let no_files = if is_tr { "⚠️ İşlenecek dosya bulunamadı." } else { "⚠️ No supported files found." };
        println!("{}", no_files.yellow());
        return Ok(());
    }

    let mut records = Vec::new();
    let geocoder = GeocodingClient::new(is_tr);
    let scanner = ExternalScanner::new(is_tr);

    let pb = if total > 1 {
        let pb = ProgressBar::new(total as u64);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")?
            .progress_chars("#>-"));
        Some(pb)
    } else {
        None
    };

    for (i, file_path) in files_to_process.iter().enumerate() {
        if let Some(ref p) = pb { p.set_message(format!("Analiz: {}", file_path)); }
        else { println!("🔍 {} {}", if is_tr { "Analiz:" } else { "Analyzing:" }.yellow(), file_path.blue().bold()); }

        let mut meta = PhotoMetadata::default();
        
        // 1. Attempt Deep Scan if requested
        if args.deep && scanner.is_exiftool_available() {
            if let Some(m) = scanner.deep_scan(file_path) {
                meta = m;
            }
        }

        // 2. Standard Scan Fallback
        if meta.latitude.is_none() {
            let mut parser = MediaParser::new();
            if let Ok(ms) = MediaSource::file_path(file_path) {
                if let Ok(iter) = parser.parse(ms) {
                    let sm = extract_metadata(iter);
                    if meta.latitude.is_none() { meta.latitude = sm.latitude; }
                    if meta.longitude.is_none() { meta.longitude = sm.longitude; }
                    if meta.altitude.is_none() { meta.altitude = sm.altitude; }
                    if meta.timestamp.is_none() { meta.timestamp = sm.timestamp; }
                    if meta.make.is_none() { meta.make = sm.make; }
                    if meta.model.is_none() { meta.model = sm.model; }
                    if meta.lens.is_none() { meta.lens = sm.lens; }
                    if meta.exposure.is_none() { meta.exposure = sm.exposure; }
                    if meta.f_number.is_none() { meta.f_number = sm.f_number; }
                    if meta.iso.is_none() { meta.iso = sm.iso; }
                }
            }
        }

        let mut address = None;
        if let (Some(lat), Some(lon)) = (meta.latitude, meta.longitude) {
            // Rate limiting for Nominatim (1 request per second)
            if i > 0 { tokio::time::sleep(Duration::from_secs(1)).await; }
            
            if let Ok(res) = geocoder.lookup(lat, lon).await {
                address = Some(res.display_name);
            }
        }

        // Output to terminal only if not in batch mode or if single file
        if pb.is_none() {
            display_metadata(&meta, is_tr);
            if let Some(ref addr) = address {
                println!("✅ {}: {}", if is_tr { "Adres" } else { "Address" }.green().bold(), addr);
                if let (Some(la), Some(lo)) = (meta.latitude, meta.longitude) {
                    println!("🔗 Google Maps: https://www.google.com/maps?q={},{}", la, lo);
                }
            }
        }

        records.push(ExportRecord::from_meta(file_path, &meta, address));
        if let Some(ref p) = pb { p.inc(1); }
    }

    if let Some(p) = pb { p.finish_with_message("Bitti!"); }

    if let Some(out_path) = args.output {
        let export_msg = if is_tr { "💾 Veriler kaydediliyor:" } else { "💾 Saving data to:" };
        println!("\n{} {}", export_msg.green().bold(), out_path.blue());
        
        let res = if out_path.ends_with(".csv") {
            save_to_csv(&records, &out_path)
        } else {
            save_to_json(&records, &out_path).map_err(|e| e.into())
        };

        match res {
            Ok(_) => println!("✅ {}", if is_tr { "Başarıyla kaydedildi." } else { "Export successful." }.green()),
            Err(e) => println!("❌ {}: {}", if is_tr { "Kayıt hatası" } else { "Export error" }.red().bold(), e),
        }
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
    
    if meta.make.is_some() || meta.model.is_some() {
        println!("\n--- {} ---", if is_tr { "TEKNİK BİLGİLER" } else { "TECHNICAL INFO" }.bold().dimmed());
        if let Some(v) = &meta.make { println!("📷 {}:   {}", labels[4].yellow(), v); }
        if let Some(v) = &meta.model { println!("📱 {}:   {}", labels[5].yellow(), v); }
        if let Some(v) = &meta.lens { println!("🔍 {}:   {}", labels[6].yellow(), v); }
        if let Some(v) = &meta.exposure { println!("⏱️ {}:{}", labels[7].yellow(), v); }
        if let Some(v) = &meta.f_number { println!("🔦 {}:{}", labels[8].yellow(), v); }
        if let Some(v) = &meta.iso { println!("🎞️ {}:     {}", labels[9].yellow(), v); }
    }
}

fn print_banner(is_tr: bool) {
    println!("{}", "===============================================".dimmed());
    println!("{}", "   GeoPic v3.5 - Professional OSINT Tool       ".bold().green());
    println!("{}", "===============================================".dimmed());
    
    if is_tr {
        println!("{}", "⚖️  GİZLİLİK VE ETİK UYARISI:".bold().yellow());
        println!("{}", "- Bu aracı sadece izniniz olan fotoğraflar için kullanın.");
        println!("{}", "- Başkalarını takip etmek veya ifşa etmek için kullanmayın.");
    } else {
        println!("{}", "⚖️  PRIVACY & ETHICS NOTICE:".bold().yellow());
        println!("{}", "- Use this tool only for ethical OSINT research.");
        println!("{}", "- Do not use for stalking or harassment.");
    }
    println!("{}\n", "===============================================".dimmed());
}
