# Multi-stage build for ultra-optimized Rust binary
FROM rust:1.75-slim AS builder

WORKDIR /app

# install deps
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# copy cargo files
COPY Cargo.toml Cargo.lock ./

# create dummy main to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm src/main.rs target/release/deps/flightsim_p2p_signaling*

# copy real source
COPY src ./src

# build optimized binary
RUN cargo build --release

# runtime image - minimal size
FROM debian:bookworm-slim

# install ca certs for HTTPS
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# create user
RUN groupadd -r signaling && useradd -r -g signaling signaling

# copy binary
COPY --from=builder /app/target/release/flightsim-p2p-signaling /usr/local/bin/signaling

# set permissions
RUN chmod +x /usr/local/bin/signaling
USER signaling

# health check removed - railway will handle this

EXPOSE 3000

CMD ["signaling"]