# Stage 1: Build the Rust binary
FROM rust:1-slim AS builder

WORKDIR /usr/src/app

# Install build tools if any dependencies require compiling C extensions
RUN apt-get update && apt-get install -y \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Copy the manifest files and source code
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY frontend ./frontend

# Compile the production binary
RUN cargo build --release --bin open-randonaut

# Stage 2: Run the binary in a clean minimal image
FROM debian:bookworm-slim

WORKDIR /app

# Copy the compiled binary
COPY --from=builder /usr/src/app/target/release/open-randonaut /app/open-randonaut

# Expose default port (overridden by PORT environment variable in production)
EXPOSE 3500

# Execute the server
CMD ["/app/open-randonaut"]
