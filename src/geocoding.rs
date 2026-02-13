use reqwest::header::USER_AGENT;
use serde::Deserialize;
use colored::*;

#[derive(Debug, Deserialize)]
pub struct NominatimResponse {
    pub display_name: String,
}

pub struct GeocodingClient {
    client: reqwest::Client,
    lang_tr: bool,
}

impl GeocodingClient {
    pub fn new(lang_tr: bool) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        
        Self { client, lang_tr }
    }

    pub async fn lookup(&self, lat: f64, lon: f64) -> Result<NominatimResponse, Box<dyn std::error::Error>> {
        let connect_msg = if self.lang_tr { 
            "🌐 Sunucuya bağlanılıyor (OpenStreetMap)..." 
        } else { 
            "🌐 Connecting to server (OpenStreetMap)..." 
        };
        println!("\n{} {}", "⚙️".dimmed(), connect_msg.dimmed());

        let url = format!(
            "https://nominatim.openstreetmap.org/reverse?format=json&lat={}&lon={}&zoom=18&addressdetails=1",
            lat, lon
        );

        let res = self.client
            .get(url)
            .header(USER_AGENT, "GeoPic-Official-Mod/3.0 (Contact: efe-rust-dev)")
            .send()
            .await?;

        if res.status().is_success() {
            let json: NominatimResponse = res.json().await?;
            Ok(json)
        } else if res.status() == 429 {
            let err_msg = if self.lang_tr { 
                "❌ Hata: API limitine takıldınız. Lütfen 1 dakika bekleyin." 
            } else { 
                "❌ Error: Rate limit hit. Please wait 1 minute." 
            };
            Err(err_msg.into())
        } else {
            Err(format!("API Error: {}", res.status()).into())
        }
    }
}
