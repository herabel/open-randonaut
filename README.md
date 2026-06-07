# 🌀 Randonautics (Quantum Anomaly Explorer)

[![Language - Rust](https://img.shields.io/badge/Language-Rust-orange?logo=rust&style=flat-square)](https://www.rust-lang.org/)
[![Backend - Axum](https://img.shields.io/badge/Backend-Axum-blue?style=flat-square)](https://github.com/tokio-rs/axum)
[![AI-Driven - Antigravity](https://img.shields.io/badge/AI--Driven-Antigravity%20AI-blueviolet?style=flat-square)](https://github.com/google-deepmind)
[![License - MIT](https://img.shields.io/badge/License-MIT-green?style=flat-square)](LICENSE)

An advanced, interactive spatial anomaly detector built to explore the relationships between intention, probability, and geographic coordinates. Drawing inspiration from quantum random coordinate generators (Randonautics), it uses continuous mathematical density algorithms to pinpoint local entropy clusters (Attractors) and void fields.

> [!NOTE]
> **🤖 AI-Driven Project**
> This codebase was created and optimized in partnership with **Antigravity**, an agentic coding assistant developed by Google DeepMind. The system design, layout optimization, continuous density algorithms, and deployment config were engineered collaboratively using pair-programming loops.

---

## 🌟 Key Features

*   **Quantum Anomaly Detection Engine**:
    *   **Attractors**: High-density point clusters representing coordinate hotspots.
    *   **Voids**: Low-density areas representing spatial blind spots.
    *   **Power Spot (Blind Spot)**: The single coordinate representing the maximum absolute Z-score anomaly.
*   **Continuous Density Estimation (KDE)**: Replaced discrete grid boundaries with a continuous **Gaussian Kernel Density Estimation (KDE)** model ($\sigma = \text{cell\_size\_m}$), eliminating z-score discretization artifacts.
*   **Centroid Localization**: Coordinates are computed as the center-of-mass (centroid) of clusters, eliminating artificial grid patterns and alignment snapping on the map.
*   **Water Exclusion Filtering**: Uses the global high-resolution (30m pixel) satellite-based ASTER database (`is-on-water` API) on the client side to check and filter coordinates falling in oceans, lakes, or wide rivers.
*   **Mind-Machine Intent Bias**: Integrates a cryptographic Argon2id key derivation function (KDF) to stretch user intention strings, mapping human consciousness queries onto quantum coordinate density pulls.
*   **Cyberpunk Mobile-Optimized UI**:
    *   Implements floating bottom sheets for settings and results on mobile screens.
    *   Fluid dark-mode styling with micro-animations.
    *   Bi-directional synchronized sliders and manual numeric inputs for search radius and point density.

---

## 🚀 Getting Started

### Prerequisites

You need the **Rust compiler** installed on your system.
If you don't have it, install it via [rustup.rs](https://rustup.rs/).

### Running Locally

1.  Clone this repository:
    ```bash
    git clone https://github.com/herabel/randonautics.git
    cd randonautics
    ```
2.  Start the Axum web server in release mode:
    ```bash
    cargo run --release --bin randonautics
    ```
3.  Open your browser and navigate to:
    [http://localhost:3500](http://localhost:3500)

---

## 📊 Performance & Stress Testing

We have included a multi-threaded stress-test benchmark binary to measure latency and scalability under load.

To run the benchmarks:
```bash
cargo run --release --bin bench
```

### Reference Benchmarks (12 vCPU Threads)
*   **Throughput (No Intent)**: ~2,880 Requests/Sec (RPS) at 4ms average latency.
*   **Throughput (With Intent/Argon2id)**: ~311 Requests/Sec (RPS) at 38ms average latency.
*   **Scaling Latencies (KDE density calculation)**:
    *   `1,024` points: **1.6ms** per session.
    *   `10,000` points: **15.0ms** per session.
    *   `65,536` points: **107.7ms** per session.
    *   `100,000` points: **145.7ms** per session.

---

## 🐳 Docker Deployment

The project contains a multi-stage Dockerfile that builds the Rust binary in a lightweight build environment and targets a minimal `debian-slim` production image (~30MB).

1.  **Build the Docker Image**:
    ```bash
    docker build -t randonautics .
    ```
2.  **Run the Container**:
    ```bash
    docker run -d -p 3500:3500 -e PORT=3500 --name randonautics-app randonautics
    ```

### Public Deployment (Render or Fly.io)

Since the application binds to the dynamic `PORT` environment variable, it is fully compatible with Fly.io or Render out-of-the-box:

```bash
# Deploy to Fly.io
fly launch
```

Deploy your static frontend to **GitHub Pages** and point it to your deployed Fly.io instance URL by modifying `API_BASE_URL` inside [frontend/index.html](frontend/index.html).

---

## 🛠 Tech Stack

*   **Backend**: [Rust](https://www.rust-lang.org/), [Axum](https://github.com/tokio-rs/axum) (HTTP router), [Tokio](https://tokio.rs/) (asynchronous engine), [Argon2](https://crates.io/crates/argon2) (KDF), [Statrs](https://crates.io/crates/statrs) (mathematical distributions).
*   **Frontend**: HTML5, Vanilla CSS3 (cyberpunk dark-theme styling), JavaScript (ES6+), Leaflet (maps library), CartoDB (dark map tiles).
*   **APIs**: `https://is-on-water.balbona.me` (ASTER Global Water Body Database).

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
