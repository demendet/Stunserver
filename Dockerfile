# Root Dockerfile for single-service Railway deployment
# This builds just the Rust signaling server for Railway

FROM rust:1.80-slim AS builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy all source files
COPY signaling/Cargo.toml ./
COPY signaling/src ./src

# Build release binary
RUN cargo build --release

# Runtime image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Create user
RUN groupadd -r signaling && useradd -r -g signaling signaling

# Copy binary
COPY --from=builder /app/target/release/flightsim-p2p-signaling /usr/local/bin/signaling
RUN chmod +x /usr/local/bin/signaling

USER signaling
EXPOSE 3000

CMD ["signaling"]