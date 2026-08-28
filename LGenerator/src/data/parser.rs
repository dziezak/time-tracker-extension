use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DomainData {
    pub name: String,
    pub seconds: u64,
    pub weight: f32, // Waga logarytmiczna (0.0 - 1.0)
    pub hue: f32,
}

pub fn parse_input_json(json_str: &str) -> Vec<DomainData> {
    let raw: HashMap<String, u64> = serde_json::from_str(json_str).unwrap_or_default();

    let max_time = raw.values().copied().max().unwrap_or(1) as f32;

    let mut result: Vec<DomainData> = raw
        .into_iter()
        .map(|(name, seconds)| {
            // Skalowanie logarytmiczne
            let log_val = (seconds as f32 + 1.0).ln();
            let log_max = (max_time + 1.0).ln();
            let weight = (log_val / log_max).clamp(0.1, 1.0);

            // Deterministyczny kolor HSL ze nazwy domeny
            let mut hash: u32 = 0;
            for byte in name.bytes() {
                hash = (byte as u32).wrapping_add((hash << 5).wrapping_sub(hash));
            }
            let hue = (hash % 360) as f32;

            DomainData { name, seconds, weight, hue }
        })
        .collect();

    result.sort_by(|a, b| b.seconds.cmp(&a.seconds));
    result
}

pub fn parse_domains() -> Vec<DomainData> {
    // Tutaj wklej kod ładowania danych z pliku JSON / wczytywania domen
    // Jeśli Twoja funkcja nazywała się inaczej, dopisz ten alias:
    vec![] // zamień na właściwe wywołanie Twojego parsera
}