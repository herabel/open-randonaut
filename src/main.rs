use axum::{
    Router,
    routing::{get, post},
    response::Html,
    Json,
};
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(serve_index))
        .route("/api/generate", post(api_generate))
        .layer(CorsLayer::permissive());

    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(3500);
    let addr = format!("0.0.0.0:{}", port);

    println!("🌀 Open Randonaut запущен → Слушает на: {}", addr);
    println!("🌐 Доступна локально: http://localhost:{}", port);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn serve_index() -> Html<&'static str> {
    Html(include_str!("../frontend/index.html"))
}

async fn api_generate(
    Json(req): Json<open_randonaut::open_randonaut::SessionRequest>,
) -> Json<open_randonaut::open_randonaut::SessionResult> {
    let result = open_randonaut::open_randonaut::generate_session(&req);
    Json(result)
}
