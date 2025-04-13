# Build stage
FROM rust:1.86 AS builder

WORKDIR /app

# Copy project files
COPY . .

# Install Leptos CLI
RUN cargo install cargo-leptos

# Install WASM target
RUN rustup target add wasm32-unknown-unknown

# Build the server + frontend with Leptos
RUN cargo leptos build --release

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy server binary
COPY --from=builder /app/target/release/leptos-full-stack .

# Copy static files
COPY --from=builder /app/target/site /app/site

# Expose port
EXPOSE 3000

CMD ["./leptos-full-stack"]
