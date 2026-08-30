use std::collections::HashMap;
use image::imageops::tile;

#[derive(Debug, Clone)]
pub struct DomainData {
    pub name: String,
    pub seconds: u64,
    pub weight: f32, // Waga logarytmiczna (0.0 - 1.0)
    pub color: [f32; 4],
}

pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [f32; 4] {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r, g, b) = match (h / 60.0) as u32 % 6 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    [r + m, g + m, b + m, 1.0]
}

pub fn get_domain_color(index: usize) -> [f32; 4] {
    let base_hues = [110.0, 45.0, 140.0, 75.0, 165.0, 90.0, 30.0];
    let hue = base_hues[index % base_hues.len()];

    hsv_to_rgb(hue, 0.65, 0.85)
}

pub fn parse_input_json(json_str: &str, top_limit: usize) -> Vec<DomainData> {
    let raw: HashMap<String, u64> = serde_json::from_str(json_str).unwrap_or_default();
    let max_time = raw.values().copied().max().unwrap_or(1) as f32;

    let mut result: Vec<DomainData> = raw
        .into_iter()
        .map(|(name, seconds)| {
            let log_val = (seconds as f32 + 1.0).ln();
            let log_max = (max_time + 1.0).ln();
            let weight = (log_val / log_max).clamp(0.2, 1.0);

            let mut hash: u32 = 0;
            for byte in name.bytes() {
                hash = (byte as u32).wrapping_add((hash << 5).wrapping_sub(hash));
            }
            let hue = (hash % 360) as f32;
            let color = hsv_to_rgb(hue, 0.75, 0.75);

            DomainData { name, seconds, weight, color }
        })
        .collect();

    result.sort_by(|a, b| b.seconds.cmp(&a.seconds));
    result.truncate(top_limit);
    result
}

pub fn parse_domains(path: &str, top_limit: usize) -> Vec<DomainData> {
    let raw_json = std::fs::read_to_string(path).unwrap();
    parse_input_json(&*raw_json, top_limit)
}