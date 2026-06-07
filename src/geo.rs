use serde::{Deserialize, Serialize};

use crate::entropy;

const EARTH_RADIUS_M: f64 = 6_371_000.0;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Coord {
    pub lat: f64,
    pub lon: f64,
}

impl Coord {
    pub fn new(lat: f64, lon: f64) -> Self {
        Self { lat, lon }
    }

    pub fn google_maps_url(&self) -> String {
        format!(
            "https://www.google.com/maps?q={},{}",
            self.lat, self.lon
        )
    }
}

/// Расстояние между двумя точками в метрах (формула Haversine)
pub fn haversine_distance(a: &Coord, b: &Coord) -> f64 {
    let d_lat = (b.lat - a.lat).to_radians();
    let d_lon = (b.lon - a.lon).to_radians();
    let lat1 = a.lat.to_radians();
    let lat2 = b.lat.to_radians();

    let h = (d_lat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (d_lon / 2.0).sin().powi(2);
    let c = 2.0 * h.sqrt().asin();

    EARTH_RADIUS_M * c
}

/// Генерация случайной точки в пределах радиуса от центра.
/// Использует 16 байт энтропии для равномерного распределения по площади круга.
pub fn random_point_in_radius(center: &Coord, radius_m: f64, entropy_bytes: &[u8; 16]) -> Coord {
    // Используем первые 8 байт для угла, вторые 8 — для расстояния
    let angle_raw = u64::from_le_bytes(entropy_bytes[0..8].try_into().unwrap());
    let dist_raw = u64::from_le_bytes(entropy_bytes[8..16].try_into().unwrap());

    let angle = (angle_raw as f64 / u64::MAX as f64) * 2.0 * std::f64::consts::PI;
    // sqrt для равномерного распределения по площади
    let dist = (dist_raw as f64 / u64::MAX as f64).sqrt() * radius_m;

    let d = dist / EARTH_RADIUS_M;
    let lat1 = center.lat.to_radians();
    let lon1 = center.lon.to_radians();

    let lat2 = (lat1.sin() * d.cos() + lat1.cos() * d.sin() * angle.cos()).asin();
    let lon2 = lon1
        + (angle.sin() * d.sin() * lat1.cos()).atan2(d.cos() - lat1.sin() * lat2.sin());

    Coord::new(lat2.to_degrees(), lon2.to_degrees())
}

/// Генерация облака случайных точек в пределах радиуса
pub fn generate_point_cloud(center: &Coord, radius_m: f64, count: usize) -> Vec<Coord> {
    let total_entropy = count * 16;
    let entropy_pool = entropy::generate_random_bytes(total_entropy);

    // Вероятностная генерация центров аномалий (аттракторов и войдов) для свободного поиска
    let mut attractors = Vec::new();
    let mut voids = Vec::new();
    if count > 32 {
        let num_att_roll = entropy_pool[0] % 100;
        let num_attractors = if num_att_roll < 35 { 1 } else if num_att_roll < 45 { 2 } else { 0 };
        for i in 0..num_attractors {
            let offset = (i + 1) * 16;
            let chunk: [u8; 16] = entropy_pool[offset..offset + 16].try_into().unwrap();
            attractors.push(random_point_in_radius(center, radius_m * 0.7, &chunk));
        }

        let num_void_roll = entropy_pool[1] % 100;
        let num_voids = if num_void_roll < 25 { 1 } else { 0 };
        for i in 0..num_voids {
            let offset = (i + 3) * 16;
            let chunk: [u8; 16] = entropy_pool[offset..offset + 16].try_into().unwrap();
            voids.push(random_point_in_radius(center, radius_m * 0.7, &chunk));
        }
    }

    let mut points = Vec::with_capacity(count);
    for i in 0..count {
        let offset = i * 16;
        let chunk: [u8; 16] = entropy_pool[offset..offset + 16].try_into().unwrap();
        let mut point = random_point_in_radius(center, radius_m, &chunk);

        // Естественное квантовое притяжение (аттракторы)
        let pull_roll = chunk[0] % 100;
        if pull_roll < 8 && !attractors.is_empty() {
            let target = attractors[0];
            let pull_factor = 0.50;
            point.lat = point.lat + (target.lat - point.lat) * pull_factor;
            point.lon = point.lon + (target.lon - point.lon) * pull_factor;
        }

        // Естественное квантовое отталкивание (войды)
        let push_roll = chunk[1] % 100;
        if push_roll < 10 && !voids.is_empty() {
            let target = voids[0];
            let push_factor = 0.50;
            let new_lat = point.lat + (point.lat - target.lat) * push_factor;
            let new_lon = point.lon + (point.lon - target.lon) * push_factor;
            let new_point = Coord::new(new_lat, new_lon);
            if haversine_distance(center, &new_point) <= radius_m {
                point = new_point;
            }
        }

        points.push(point);
    }
    points
}

/// Генерация облака точек с гибридом энтропия + хеш намерения (XOR) и вероятностными аномалиями
pub fn generate_point_cloud_with_intent(
    center: &Coord,
    radius_m: f64,
    count: usize,
    intent_hash: &[u8; 32],
) -> Vec<Coord> {
    let total_entropy = count * 16;
    let mut entropy_pool = entropy::generate_random_bytes(total_entropy);

    // XOR энтропию с хешем намерения (циклически)
    for (i, byte) in entropy_pool.iter_mut().enumerate() {
        *byte ^= intent_hash[i % 32];
    }

    // Вероятностная генерация на основе хеша намерения (сознание не всегда рождает аномалии)
    // Генерируем от 0 до 3 аттракторов и от 0 до 2 войдов
    let num_attractors = (intent_hash[0] % 4) as usize; // 0, 1, 2 или 3 аттрактора
    let num_voids = (intent_hash[1] % 3) as usize;       // 0, 1 или 2 войда

    let mut attractors = Vec::new();
    for i in 0..num_attractors {
        let b_idx = i * 4;
        let mut chunk = [0u8; 16];
        for j in 0..16 {
            chunk[j] = intent_hash[(b_idx + j) % 32];
        }
        let att = random_point_in_radius(center, radius_m * 0.7, &chunk);
        attractors.push(att);
    }

    let mut voids = Vec::new();
    for i in 0..num_voids {
        let b_idx = 16 + i * 4;
        let mut chunk = [0u8; 16];
        for j in 0..16 {
            chunk[j] = intent_hash[(b_idx + j) % 32];
        }
        let vd = random_point_in_radius(center, radius_m * 0.7, &chunk);
        voids.push(vd);
    }

    let mut points = Vec::with_capacity(count);
    for i in 0..count {
        let offset = i * 16;
        let chunk: [u8; 16] = entropy_pool[offset..offset + 16].try_into().unwrap();
        let mut point = random_point_in_radius(center, radius_m, &chunk);

        // Влияние намерения разума: стягиваем 15% точек к одному из аттракторов
        let pull_roll = chunk[0] % 100;
        if pull_roll < 15 && !attractors.is_empty() {
            let att_idx = (chunk[1] as usize) % attractors.len();
            let target = attractors[att_idx];
            let pull_factor = 0.75;
            point.lat = point.lat + (target.lat - point.lat) * pull_factor;
            point.lon = point.lon + (target.lon - point.lon) * pull_factor;
        }

        // Влияние намерения разума: отталкиваем 15% точек от одного из войдов
        let push_roll = chunk[2] % 100;
        if push_roll < 15 && !voids.is_empty() {
            let vd_idx = (chunk[3] as usize) % voids.len();
            let target = voids[vd_idx];
            let push_factor = 0.60;
            let new_lat = point.lat + (point.lat - target.lat) * push_factor;
            let new_lon = point.lon + (point.lon - target.lon) * push_factor;
            let new_point = Coord::new(new_lat, new_lon);
            if haversine_distance(center, &new_point) <= radius_m {
                point = new_point;
            }
        }

        points.push(point);
    }
    points
}
