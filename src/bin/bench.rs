use randonautics::randonautics::{generate_session, SessionRequest};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

fn print_latency_stats(latencies: &mut [Duration]) {
    if latencies.is_empty() {
        return;
    }
    latencies.sort();
    let total = latencies.len();
    let sum: Duration = latencies.iter().sum();
    let avg = sum / total as u32;
    let min = latencies[0];
    let max = latencies[total - 1];
    let p50 = latencies[total / 2];
    let p95 = latencies[((total as f64 * 0.95) as usize).min(total - 1)];
    let p99 = latencies[((total as f64 * 0.99) as usize).min(total - 1)];
    
    println!("  ⏱️  Распределение задержек (Latency):");
    println!("      - Среднее время:   {:?}", avg);
    println!("      - Минимальное:     {:?}", min);
    println!("      - p50 (Медиана):   {:?}", p50);
    println!("      - p95 (95% линий): {:?}", p95);
    println!("      - p99 (99% линий): {:?}", p99);
    println!("      - Максимальное:    {:?}", max);
}

fn main() {
    println!("🚀 Запуск расширенного бенчмарка Рандонавтики");
    println!("Оборудование: Многопоточный стресс-тест & Замеры масштабируемости");

    let num_threads = thread::available_parallelism()
        .map(|p| p.get())
        .unwrap_or(12);

    println!("Используемых потоков: {}", num_threads);

    let test_duration = Duration::from_secs(5);

    // ==========================================
    // ТЕСТ 1: БЕЗ НАМЕРЕНИЯ (Чистая энтропия)
    // ==========================================
    println!("\n▶ Тест 1: Запросы БЕЗ намерения (чистая энтропия + геометрия + поиск аномалий, 1024 точки)");
    let counter_no_intent = Arc::new(AtomicUsize::new(0));

    let mut threads = vec![];
    let start_time = Instant::now();

    for _ in 0..num_threads {
        let counter = Arc::clone(&counter_no_intent);
        threads.push(thread::spawn(move || {
            let mut thread_latencies = Vec::new();
            let req = SessionRequest {
                lat: 55.7558,
                lon: 37.6173,
                radius: 3000.0,
                point_count: 1024,
                intent: None,
            };
            while start_time.elapsed() < test_duration {
                let session_start = Instant::now();
                let _ = generate_session(&req);
                thread_latencies.push(session_start.elapsed());
                counter.fetch_add(1, Ordering::Relaxed);
            }
            thread_latencies
        }));
    }

    let mut all_latencies_no_intent = Vec::new();
    for t in threads {
        if let Ok(mut latencies) = t.join() {
            all_latencies_no_intent.append(&mut latencies);
        }
    }

    let total_no_intent = counter_no_intent.load(Ordering::Relaxed);
    let rps_no_intent = total_no_intent as f64 / test_duration.as_secs_f64();
    println!("✅ Результат: {:.2} запросов в секунду (RPS)", rps_no_intent);
    println!("Всего сгенерировано: {} сессий за {} секунд", total_no_intent, test_duration.as_secs());
    print_latency_stats(&mut all_latencies_no_intent);

    // ==========================================
    // ТЕСТ 2: С НАМЕРЕНИЕМ (Argon2id + Энтропия)
    // ==========================================
    println!("\n▶ Тест 2: Запросы С намерением (дополнительно работает криптография Argon2id)");
    let counter_intent = Arc::new(AtomicUsize::new(0));

    let mut threads = vec![];
    let start_time = Instant::now();

    for i in 0..num_threads {
        let counter = Arc::clone(&counter_intent);
        threads.push(thread::spawn(move || {
            let mut thread_latencies = Vec::new();
            let req = SessionRequest {
                lat: 55.7558,
                lon: 37.6173,
                radius: 3000.0,
                point_count: 1024,
                intent: Some(format!("намерение_{}", i)),
            };
            while start_time.elapsed() < test_duration {
                let session_start = Instant::now();
                let _ = generate_session(&req);
                thread_latencies.push(session_start.elapsed());
                counter.fetch_add(1, Ordering::Relaxed);
            }
            thread_latencies
        }));
    }

    let mut all_latencies_intent = Vec::new();
    for t in threads {
        if let Ok(mut latencies) = t.join() {
            all_latencies_intent.append(&mut latencies);
        }
    }

    let total_intent = counter_intent.load(Ordering::Relaxed);
    let rps_intent = total_intent as f64 / test_duration.as_secs_f64();
    println!("✅ Результат: {:.2} запросов в секунду (RPS)", rps_intent);
    println!("Всего сгенерировано: {} сессий за {} секунд", total_intent, test_duration.as_secs());
    print_latency_stats(&mut all_latencies_intent);

    // ==========================================
    // ТЕСТ 3: ЗАМЕРЫ МАСШТАБИРУЕМОСТИ ПО ТОЧКАМ
    // ==========================================
    println!("\n▶ Тест 3: Замеры масштабируемости (время обработки одной сессии в зависимости от количества точек)");
    let point_counts = [1024, 10_000, 65_536, 100_000];
    for &pc in &point_counts {
        let req = SessionRequest {
            lat: 55.7558,
            lon: 37.6173,
            radius: 3000.0,
            point_count: pc,
            intent: None,
        };
        // Для больших объемов делаем меньше итераций, чтобы тест не шел слишком долго
        let iterations = if pc > 10_000 { 5 } else { 20 };
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = generate_session(&req);
        }
        let elapsed = start.elapsed() / iterations as u32;
        println!("  - {: >7} точек: {:?}", pc, elapsed);
    }

    // ==========================================
    // ВЫВОДЫ
    // ==========================================
    println!("\n📊 Оценка вместимости сервера:");
    println!("При смешанной нагрузке (например, 20% с намерением, 80% без намерений):");
    
    let mixed_rps = 1.0 / (0.8 / rps_no_intent + 0.2 / rps_intent);
    println!("~ {:.0} запросов в секунду сервер потянет без проблем.", mixed_rps);
    println!("Если запрос делает пользователь раз в минуту, это {MAX_USERS} одновременных активных пользователей онлайн.", MAX_USERS = (mixed_rps * 60.0) as u32);
}
