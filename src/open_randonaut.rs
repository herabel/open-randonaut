use serde::Serialize;

use crate::geo::{self, Coord};
use crate::intent;

/// Тип аномалии
#[derive(Debug, Clone, Copy, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AnomalyType {
    Attractor,
    Void,
    Power,
}

/// Найденная аномалия
#[derive(Debug, Clone, Serialize)]
pub struct Anomaly {
    #[serde(rename = "type")]
    pub kind: AnomalyType,
    pub coord: Coord,
    pub z_score: f64,
    pub point_count: usize,
    pub google_maps_url: String,
}

/// Ячейка сетки для анализа плотности
#[derive(Debug)]
struct GridCell {
    lat_center: f64,
    lon_center: f64,
    count: usize,
    sum_lat: f64,
    sum_lon: f64,
}

impl GridCell {
    fn coord(&self) -> Coord {
        if self.count > 0 {
            Coord::new(self.sum_lat / self.count as f64, self.sum_lon / self.count as f64)
        } else {
            Coord::new(self.lat_center, self.lon_center)
        }
    }
}

/// Параметры сессии
#[derive(Debug, Clone, serde::Deserialize)]
pub struct SessionRequest {
    pub lat: f64,
    pub lon: f64,
    pub radius: f64,
    #[serde(default = "default_point_count")]
    pub point_count: usize,
    #[serde(default)]
    pub intent: Option<String>,
}

fn default_point_count() -> usize {
    1024
}

/// Результат сессии
#[derive(Debug, Serialize)]
pub struct SessionResult {
    pub center: Coord,
    pub radius: f64,
    pub point_count: usize,
    pub has_intent: bool,
    pub anomalies: Vec<Anomaly>,
    pub points: Vec<Coord>,
    pub entropy_sources: Vec<String>,
}

/// Z-score порог для аномалий
const Z_THRESHOLD: f64 = 2.5;

/// Минимальное количество ячеек сетки по одному измерению
const MIN_GRID_SIZE: usize = 16;
/// Максимальное количество ячеек сетки по одному измерению
const MAX_GRID_SIZE: usize = 80;

/// Определяет размер сетки в зависимости от количества точек
fn compute_grid_size(point_count: usize) -> usize {
    // Оптимальное масштабирование сетки (sqrt(N) * 1.2) для баланса разрешения и статистической плотности
    let raw = ((point_count as f64).sqrt() * 1.2) as usize;
    raw.clamp(MIN_GRID_SIZE, MAX_GRID_SIZE)
}

/// Находит аномалии в облаке точек методом сеточного анализа плотности
pub fn find_anomalies(points: &[Coord], center: &Coord, radius_m: f64) -> Vec<Anomaly> {
    let grid_size = compute_grid_size(points.len());

    // Вычисляем bounding box
    let lat_delta = (radius_m / 111_320.0).abs();
    let cos_lat = center.lat.to_radians().cos().max(0.0001);
    let lon_delta = (radius_m / (111_320.0 * cos_lat)).abs();

    let lat_min = center.lat - lat_delta;
    let lat_max = center.lat + lat_delta;
    let lon_min = center.lon - lon_delta;
    let lon_max = center.lon + lon_delta;

    let lat_step = (lat_max - lat_min) / grid_size as f64;
    let lon_step = (lon_max - lon_min) / grid_size as f64;

    // Создаём сетку и считаем точки в каждой ячейке
    let total_cells = grid_size * grid_size;
    let mut cells: Vec<GridCell> = Vec::with_capacity(total_cells);
    for row in 0..grid_size {
        for col in 0..grid_size {
            cells.push(GridCell {
                lat_center: lat_min + (row as f64 + 0.5) * lat_step,
                lon_center: lon_min + (col as f64 + 0.5) * lon_step,
                count: 0,
                sum_lat: 0.0,
                sum_lon: 0.0,
            });
        }
    }

    // Распределяем точки по ячейкам и накапливаем координаты для вычисления центроидов
    for point in points {
        let row = ((point.lat - lat_min) / lat_step) as usize;
        let col = ((point.lon - lon_min) / lon_step) as usize;
        if row < grid_size && col < grid_size {
            let idx = row * grid_size + col;
            cells[idx].count += 1;
            cells[idx].sum_lat += point.lat;
            cells[idx].sum_lon += point.lon;
        }
    }

    // Вычисляем сглаженную плотность (KDE) для каждой ячейки
    let mut weights = vec![0.0f64; total_cells];
    let cell_size_m = (2.0 * radius_m) / grid_size as f64;
    let sigma = cell_size_m;
    let two_sigma_sq = 2.0 * sigma * sigma;

    for point in points {
        let row_float = (point.lat - lat_min) / lat_step;
        let col_float = (point.lon - lon_min) / lon_step;
        let row_center = row_float.round() as isize;
        let col_center = col_float.round() as isize;

        let r_range = 3;
        for dr in -r_range..=r_range {
            let r = row_center + dr;
            if r < 0 || r >= grid_size as isize {
                continue;
            }
            let r = r as usize;

            for dc in -r_range..=r_range {
                let c = col_center + dc;
                if c < 0 || c >= grid_size as isize {
                    continue;
                }
                let c = c as usize;

                let cell = &cells[r * grid_size + c];
                let cell_coord = Coord::new(cell.lat_center, cell.lon_center);
                let d = geo::haversine_distance(&cell_coord, point);

                let w = (-d * d / two_sigma_sq).exp();
                weights[r * grid_size + c] += w;
            }
        }
    }

    // Исключаем краевую буферную зону (2.0 * cell_size_m), чтобы предотвратить краевые эффекты (кольцо войдов на границе)
    let limit_dist = radius_m - 2.0 * cell_size_m;

    // Вычисляем среднюю плотность и стандартное отклонение только для ячеек внутри радиуса (с буфером)
    let mut active_weights = Vec::with_capacity(total_cells);
    for (i, cell) in cells.iter().enumerate() {
        let cell_coord = cell.coord();
        if geo::haversine_distance(center, &cell_coord) <= limit_dist {
            active_weights.push(weights[i]);
        }
    }

    if active_weights.is_empty() {
        return vec![];
    }

    let mean = active_weights.iter().sum::<f64>() / active_weights.len() as f64;
    let variance = active_weights.iter().map(|w| (w - mean).powi(2)).sum::<f64>() / active_weights.len() as f64;
    let stddev = variance.sqrt();

    if stddev < 1e-10 {
        // Все ячейки одинаковые — нет аномалий
        return vec![];
    }

    // Z-scores и детекция аномалий
    let mut anomalies = Vec::new();
    let mut max_abs_z: f64 = 0.0;
    let mut power_idx: Option<usize> = None;

    for (i, cell) in cells.iter().enumerate() {
        let z = (weights[i] - mean) / stddev;
        let cell_coord = cell.coord();

        // Проверяем, что координата ячейки находится строго в пределах радиуса (с учётом буфера)
        if geo::haversine_distance(center, &cell_coord) <= limit_dist {
            if z > Z_THRESHOLD {
                anomalies.push(Anomaly {
                    kind: AnomalyType::Attractor,
                    coord: cell_coord,
                    z_score: (z * 100.0).round() / 100.0,
                    point_count: cell.count,
                    google_maps_url: cell_coord.google_maps_url(),
                });
            } else if z < -Z_THRESHOLD {
                anomalies.push(Anomaly {
                    kind: AnomalyType::Void,
                    coord: cell_coord,
                    z_score: (z * 100.0).round() / 100.0,
                    point_count: cell.count,
                    google_maps_url: cell_coord.google_maps_url(),
                });
            }

            if cell.count > 0 && z.abs() > max_abs_z {
                max_abs_z = z.abs();
                power_idx = Some(i);
            }
        }
    }

    // Power (Blind Spot) — точка с максимальным |z-score|
    if let Some(idx) = power_idx {
        let cell = &cells[idx];
        let z = (weights[idx] - mean) / stddev;
        let cell_coord = cell.coord();
        let power = Anomaly {
            kind: AnomalyType::Power,
            coord: cell_coord,
            z_score: (z * 100.0).round() / 100.0,
            point_count: cell.count,
            google_maps_url: cell_coord.google_maps_url(),
        };
        // Добавляем Power только если его нет среди уже найденных (с теми же координатами)
        let already_exists = anomalies.iter().any(|a| {
            (a.coord.lat - power.coord.lat).abs() < 1e-10
                && (a.coord.lon - power.coord.lon).abs() < 1e-10
        });
        if !already_exists {
            anomalies.push(power);
        }
    }

    // Сортируем по |z-score| — сильнейшие аномалии первыми
    anomalies.sort_by(|a, b| {
        b.z_score
            .abs()
            .partial_cmp(&a.z_score.abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Применяем Non-Maximum Suppression (NMS) для объединения близких аномалий одного типа
    let cell_size_m = (2.0 * radius_m) / grid_size as f64;
    let merge_radius = 1.5 * cell_size_m;

    let mut selected_anomalies: Vec<Anomaly> = Vec::new();
    for anomaly in anomalies {
        let mut is_suppressed = false;
        for selected in &selected_anomalies {
            if anomaly.kind == selected.kind {
                let dist = geo::haversine_distance(&anomaly.coord, &selected.coord);
                if dist < merge_radius {
                    is_suppressed = true;
                    break;
                }
            }
        }
        if !is_suppressed {
            selected_anomalies.push(anomaly);
        }
    }

    selected_anomalies
}

/// Провести полную сессию рандонавтики
pub fn generate_session(req: &SessionRequest) -> SessionResult {
    let center = Coord::new(req.lat, req.lon);
    let count = req.point_count.clamp(64, 100_000);

    let has_intent = req
        .intent
        .as_ref()
        .is_some_and(|s| !s.trim().is_empty());

    let points = if has_intent {
        let intent_hash = intent::hash_intent(req.intent.as_deref().unwrap());
        geo::generate_point_cloud_with_intent(&center, req.radius, count, &intent_hash)
    } else {
        geo::generate_point_cloud(&center, req.radius, count)
    };

    let anomalies = find_anomalies(&points, &center, req.radius);

    let mut entropy_sources = vec!["OS (getrandom)".to_string(), "SHAKE256".to_string()];
    if crate::cpu_entropy::gen_rdrand(1).is_some() {
        entropy_sources.push("RDRAND".to_string());
    }
    if crate::cpu_entropy::gen_rdseed(1).is_some() {
        entropy_sources.push("RDSEED".to_string());
    }
    if has_intent {
        entropy_sources.push("Argon2id (intent)".to_string());
    }

    SessionResult {
        center,
        radius: req.radius,
        point_count: count,
        has_intent,
        anomalies,
        points,
        entropy_sources,
    }
}
