# Root Dockerfile for single-service Railway deployment
# This builds just the Rust signaling server for Railway

FROM rust:1.75-slim AS builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy signaling server source
COPY signaling/Cargo.toml signaling/Cargo.lock ./
COPY signaling/src ./src

# Create dummy main to cache dependencies
RUN mkdir -p src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm src/main.rs target/release/deps/flightsim_p2p_signaling*

# Copy real source and build
COPY signaling/src ./src
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