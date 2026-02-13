use std::process::Command;
use serde_json::Value;
use crate::metadata::PhotoMetadata;
use colored::*;

pub struct ExternalScanner {
    is_tr: bool,
}

impl ExternalScanner {
    pub fn new(is_tr: bool) -> Self {
        Self { is_tr }
    }

    /// Checks if exiftool is installed on the system
    pub fn is_exiftool_available(&self) -> bool {
        Command::new("exiftool")
            .arg("-ver")
            .output()
            .is_ok()
    }

    /// Performs a deep scan using exiftool
    pub fn deep_scan(&self, file_path: &str) -> Option<PhotoMetadata> {
        let output = Command::new("exiftool")
            .args(["-json", "-GPSLatitude", "-GPSLongitude", "-GPSAltitude", "-Make", "-Model", "-LensModel", "-ExposureTime", "-FNumber", "-ISO", file_path])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let json_str = String::from_utf8_lossy(&output.stdout);
        let v: Value = serde_json::from_str(&json_str).ok()?;
        let item = v.as_array()?.first()?;

        let mut meta = PhotoMetadata::default();
        
        // Extracting data from JSON
        if let Some(lat) = item.get("GPSLatitude").and_then(|l| l.as_f64()) { meta.latitude = Some(lat); }
        if let Some(lon) = item.get("GPSLongitude").and_then(|l| l.as_f64()) { meta.longitude = Some(lon); }
        if let Some(alt) = item.get("GPSAltitude").and_then(|a| a.as_f64()) { meta.altitude = Some(alt); }
        
        meta.make = item.get("Make").and_then(|m| m.as_str()).map(|s| s.to_string());
        meta.model = item.get("Model").and_then(|m| m.as_str()).map(|s| s.to_string());
        meta.lens = item.get("LensModel").and_then(|l| l.as_str()).map(|s| s.to_string());
        meta.exposure = item.get("ExposureTime").and_then(|e| e.as_str()).map(|s| s.to_string());
        meta.f_number = item.get("FNumber").and_then(|f| f.as_f64()).map(|n| format!("{:.1}", n));
        meta.iso = item.get("ISO").and_then(|i| i.as_i64()).map(|n| n.to_string());

        Some(meta)
    }

    pub fn print_dependency_warning(&self) {
        if self.is_tr {
            println!("\n{} {}", "⚠️".yellow(), "Uyarı: 'exiftool' sistemi üzerinde bulunamadı.".bold());
            println!("{}", "Derin tarama için lütfen yükleyin: 'sudo apt install libimage-exiftool-perl'".dimmed());
        } else {
            println!("\n{} {}", "⚠️".yellow(), "Warning: 'exiftool' not found on system.".bold());
            println!("{}", "Please install it for deep scan support: 'sudo apt install libimage-exiftool-perl'".dimmed());
        }
    }
}
